use std::ffi::c_void;
use std::mem::align_of;
use std::ptr::{from_mut, from_ref};
use std::slice;

use super::DebugFilter;
use crate::bindings::types::*;
use crate::bindings::{DONT_CARE as GL_DONT_CARE, MAX_DEBUG_MESSAGE_LENGTH as GL_MAX_DEBUG_MESSAGE_LENGTH};
use crate::types::{DebugMessage, DebugSeverity, DebugSource, DebugType};
use crate::{convert, GLContext};


/// Copies a message returned by OpenGL into an owned Rust [`String`]. Panics if any invariants are unmet.
unsafe fn msg_to_string(str: *const GLchar, len: GLsizei) -> String {
    // From OpenGL Spec 4.6 section 20.2:
    // > The memory for `message` is owned and managed by the GL, and should only be considered valid for the
    // > duration of the [callback].
    // Easiest thing to do is just to copy it into a plain Rust string.

    // Validate `str` and `len` before converting to slice
    let len: usize = len.try_into().expect("GL debug msg len must be 0 <= n <= isize::MAX");
    assert!(len <= (isize::MAX as usize), "GL debug msg len must be 0 <= n <= isize::MAX");
    assert!(!str.is_null(), "GL debug msg str must be non-null");
    assert!((str as usize).checked_add(len).is_some(), "GL debug msg str must not wrap address space");
    assert!(str.align_offset(align_of::<GLchar>()) == 0, "GL debug msg str must be aligned to GLchar");

    // SAFETY:
    // - Just validated length, wrapping, alignment, and non-null requirements
    // - We trust OpenGL to provide a string within a single allocated object
    // - OpenGL spec asserts that the message is valid for the duration of the callback (this function), long
    //   enough to read from
    // - We do not mutate the slice, only copy from it.
    let msg_slice: &[GLchar] = unsafe { slice::from_raw_parts(str, len) };

    // Hand-rolled "lossy" conversion into UTF-8 string
    msg_slice
        .into_iter()
        .map(|&c| {
            u32::try_from(c)
                .ok()
                .and_then(|u| char::from_u32(u))
                .unwrap_or(char::REPLACEMENT_CHARACTER)
        })
        .collect()
}


impl GLContext {
    /// Untested.
    pub fn debug_message_control(&self, filter: DebugFilter, enabled: bool) {
        let enabled = convert!(enabled, GLboolean, "'enabled' boolean");
        let (src, typ, sev, count, ids) = match filter {
            DebugFilter::Where { source: src, typ, severity: sev } => (
                src.map(|e| e.into_raw()).unwrap_or(GL_DONT_CARE),
                typ.map(|e| e.into_raw()).unwrap_or(GL_DONT_CARE),
                sev.map(|e| e.into_raw()).unwrap_or(GL_DONT_CARE),
                0,
                std::ptr::null(),
            ),
            DebugFilter::ById { source: src, typ, ids } => (
                src.into_raw(),
                typ.into_raw(),
                GL_DONT_CARE,
                convert!(ids.len(), GLsizei, "number of debug filter IDs"),
                from_ref(ids).cast(),
            ),
        };

        unsafe { self.gl.debug_message_control(src, typ, sev, count, ids, enabled) }
    }

    /// WIP/untested. Contains a lot of unsafe code, so it should be tested.
    pub fn get_debug_message_log(&self, count: usize) -> Vec<DebugMessage> {
        // Borrows pretty heavily from the example at: https://www.khronos.org/opengl/wiki/Debug_Output.

        // TODO: actually implement `glGet*`, with some nice enum stuff (or triaits!!)
        let max_msg_len = {
            let mut n: GLint = 0;
            unsafe {
                self.gl.get_integer_v(GL_MAX_DEBUG_MESSAGE_LENGTH, from_mut(&mut n));
            }

            if n <= 0 {
                // should never happen, but w/e
                return vec![];
            } else {
                n as usize
            }
        };

        let mut text_data: Vec<GLchar> = Vec::with_capacity(count * max_msg_len);
        let mut src_data: Vec<GLenum> = Vec::with_capacity(count);
        let mut typ_data: Vec<GLenum> = Vec::with_capacity(count);
        let mut sev_data: Vec<GLenum> = Vec::with_capacity(count);
        let mut id_data: Vec<GLuint> = Vec::with_capacity(count);
        let mut lengths: Vec<GLsizei> = Vec::with_capacity(count);

        let count = convert!(count, GLuint, "number of debug message logs");
        let buf_size = convert!(text_data.capacity(), GLsizei, "total buffer size for debug message data");

        let num_found = unsafe {
            self.gl.get_debug_message_log(
                count,
                buf_size,
                src_data.as_mut_ptr(),
                typ_data.as_mut_ptr(),
                id_data.as_mut_ptr(),
                sev_data.as_mut_ptr(),
                lengths.as_mut_ptr(),
                text_data.as_mut_ptr(),
            )
        } as usize;

        // Once we know how many were put into the arrays behind the scenes, resize them to match.
        // SAFETY: OpenGL spec guarantees that it will not write more than `count` items into the given vectors.
        unsafe {
            src_data.set_len(num_found);
            typ_data.set_len(num_found);
            sev_data.set_len(num_found);
            id_data.set_len(num_found);
            lengths.set_len(num_found);
        };

        // Pointer into character data that we're going to inch along and copy out of.
        let mut str_ptr: *const GLchar = text_data.as_ptr();
        let mut messages = Vec::with_capacity(num_found);

        for msg in 0..num_found {
            let id = id_data[msg];
            let source = DebugSource::from_raw(src_data[msg]).expect("OpenGL should return a valid DebugSource");
            let severity = DebugSeverity::from_raw(sev_data[msg]).expect("OpenGL should return a valid DebugSeverity");
            let typ = DebugType::from_raw(typ_data[msg]).expect("OpenGL should return a valid DebugType");

            let str_len: GLsizei = lengths[msg];

            // SAFETY: The pointer comes from a Rust-allocated object, so it is known to be valid (does not overflow
            // isize, does not wrap, etc). `msg_to_string` does the rest of the required checks. First statement goes
            // for `byte_add` as well.
            let body = unsafe { msg_to_string(str_ptr, str_len) };
            str_ptr = unsafe { str_ptr.byte_add(str_len as usize) };

            messages.push(DebugMessage { id, typ, source, severity, body });
        }

        messages
    }

    /// This function requires a mutable reference to the context because the callback closure, if it captures anything,
    /// must be kept alive for as long as OpenGL may call the debug callback. If this function is called twice, the
    /// callback from the first call is replaced and its closure is dropped.
    ///
    /// [`glDebugMessageCallback`]: https://registry.khronos.org/OpenGL-Refpages/gl4/html/glDebugMessageCallback.xhtml
    pub fn debug_message_callback<F: FnMut(DebugMessage) + Sync + 'static>(&mut self, callback: F) {
        /// This is the function that actually receives OpenGL's callback.
        extern "system" fn intercept_debug_callback<C: FnMut(DebugMessage) + Sync + 'static>(
            src: GLenum,             // GLenum source
            typ: GLenum,             // GLenum type
            id: GLuint,              // GLuint id
            sev: GLenum,             // GLenum severity
            msg_len: GLsizei,        // GLsizei length
            msg_str: *const GLchar,  // const GLchar* message
            user_param: *mut c_void, // const void* userParam
        ) {
            let body = unsafe { msg_to_string(msg_str, msg_len) };

            // Check before we cast our pointer and call a method from it willy-nilly:
            assert!(!user_param.is_null(), "GL debug callback: pointer is null");
            assert!(user_param.align_offset(align_of::<C>()) == 0, "GL debug callback: pointer misaligned");

            // SAFETY:
            // - Alignment and non-null have been asserted.
            // - The memory (closure) living in the box will remain valid until the box is dropped; the box will not be
            //   dropped unless the struct is dropped; GLContext's drop impl calls `unset_debug_message_callback`, so if
            //   it _is_ dropped then this function will be unset as the callback and it won't be called.
            // - Since the `intercept_callback` function is private, the only time this function will ever be the
            //   callback is when we set it to be; since the only time we do that has the correct `userParam` value, we
            //   know it'll be the closure type we want.
            // - The only exception to the last point is that it is technically possible for the user to
            let closure = unsafe { (user_param as *mut C).as_mut().unwrap_unchecked() };

            // Note to self: Even though my mind keeps overthinking of ways that ACE could happen, it's not really that
            // big of a concern, at least here. Since OpenGL lets you query the current state of the `callback` and
            // `userParam` values (with `glGetPointerv`), an attacker could theoretically gain access to the pointer to
            // this wrapper function and reconfigure OpenGL to call it with a malicious `userParam`, resulting in us
            // jumping execution elsewhere. However:
            //
            // - Wherever we end up jumping to does not receive any highly sensitive information, at least readily on
            //   the stack; the only thing we pass is the message itself.
            // - **This is nothing that the attacker could not already do by calling `glDebugMessageCallback` on their
            //   own anyways.** _That_ function is the vulnerable entrypoint--this doesn't make it any more unsafe than
            //   it already was.

            closure(DebugMessage {
                id,
                typ: DebugType::from_raw(typ).unwrap_or(DebugType::Other),
                source: DebugSource::from_raw(src).unwrap_or(DebugSource::Other),
                severity: DebugSeverity::from_raw(sev).unwrap_or(DebugSeverity::High), // Default of High seems sensible
                body,
            });
        }

        let mut callback: Box<F> = Box::new(callback); // Move closure to the heap so it doesn't move
        let cb_ptr: *mut F = from_mut(&mut *callback); // Get a raw pointer to the box's contents
        let fn_ptr = intercept_debug_callback::<F>; // fn() pointer instantiated specifically with this closure type
        unsafe {
            self.gl.debug_message_callback(Some(fn_ptr), cb_ptr.cast());
        }

        // Save the callback. As long as this struct stays alive, the Box will be valid and the closure will not be
        // removed.
        self.debug_callback = Some(callback);
    }

    /// Unsets the debug message callback which was previously established by [`Self::debug_message_callback`].
    ///
    /// If the previous callback was an owning closure, it will be dropped after this function is called.
    pub fn unset_debug_message_callback(&mut self) {
        unsafe {
            self.gl.debug_message_callback(None, std::ptr::null());
        }

        // Ensure we only perform the drop once we actually do the FFI call that takes the pointer away from OpenGL.
        // Otherwise, OpenGL may attempt to call into the function that no longer exists!
        self.debug_callback = None;
    }
}

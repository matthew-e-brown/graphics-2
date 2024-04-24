//! Raw OpenGL bindings that map directly to `extern` function calls.
//!
//! Most of this module's functionality lives within the [`GLPointers`] struct.

use core::ffi::{c_char, c_void};

use types::*;

type VoidPtr = *const c_void;

/// FFI-style type aliases that match with the types used by OpenGL, such as `GLint`, `GLboolean`, `GLenum`, and so on.
///
/// These types are defined in table 2.2 on page 13 of the 4.6 core specification. They are specifically _not_ C types,
/// and have hard-coded sizes. Some underlying types have multiple aliases to provide a semantic distinction. For
/// example, [`GLsizei`] and [`GLint`] are equivalent, but one is used to denote _sizes_ (of buffers, etc.), while the
/// other is a simple integer.
pub mod types {
    use super::c_char;

    // Standard type aliases
    // -----------------------------------------------------------------------------

    /// Boolean.
    pub type GLboolean = u8;

    /// Signed two's complement binary integer.
    pub type GLbyte = i8;

    /// Unsigned binary integer.
    pub type GLubyte = u8;

    /// Characters making up strings.
    pub type GLchar = c_char;
    // Use the FFI type instead of `u8` or `i8` directly since the signedness of a "char" will likely differ by target.

    /// Signed two's complement binary integer.
    pub type GLshort = i16;

    /// Unsigned binary integer.
    pub type GLushort = u16;

    /// Signed two's complement binary integer.
    pub type GLint = i32;

    /// Unsigned binary integer.
    pub type GLuint = u32;

    /// Signed two's complement 16.16 scaled integer.
    pub type GLfixed = i32;

    /// Signed two's complement binary integer.
    pub type GLint64 = i64;

    /// Unsigned binary integer.
    pub type GLuint64 = u64;

    /// Non-negative binary integer size.
    ///
    /// Since the OpenGL spec calls this (and some other types) "non-negative binary integer size," you would expect it
    /// to be a u32; however, the raw `gl.xml` uses a plain `int` in its typedef instead of `unsigned int` or
    /// `uint32_t`. We do the same for consistency.
    pub type GLsizei = i32;

    /// Enumerated binary integer value.
    pub type GLenum = u32;

    /// Signed two's complement binary integer (`ptrbits`).
    pub type GLintptr = isize;

    /// Non-negative binary integer size (`ptrbits`).
    pub type GLsizeiptr = isize;

    /// Sync object handle.
    pub type GLsync = *const GLSyncHandle;

    /// Bit field.
    pub type GLbitfield = u32;

    /// Half-precision floating-point value encoding in an unsigned scalar.
    pub type GLhalf = u16;

    /// Floating-point value.
    pub type GLfloat = f32;

    /// Floating-point value clamped to [0, 1].
    pub type GLclampf = f32;

    /// Floating-point value.
    pub type GLdouble = f64;

    /// Floating-point value clamped to [0, 1].
    pub type GLclampd = f64;

    // Opaque types, serve as pointees (things that are pointed to) in some other types.
    // -----------------------------------------------------------------------------

    /// Opaque type. Used as a pointee.
    pub enum GLSyncHandle {}

    /// Opaque type. Compatible with OpenCL `cl_context`.
    pub enum CLContext {}

    /// Opaque type. Compatible with OpenCL `cl_event`.
    pub enum CLEvent {}

    // Function pointers, used for callbacks.
    // -----------------------------------------------------------------------------

    pub type GLDebugProc = Option<
        extern "system" fn(
            source: GLenum,
            gl_type: GLenum,
            id: GLuint,
            severity: GLenum,
            length: GLsizei,
            message: *const GLchar,
            user_param: *mut core::ffi::c_void,
        ),
    >;

    // Vendor extension types
    // -----------------------------------------------------------------------------

    #[allow(non_camel_case_types)]
    pub type GLDebugProc_AMD = Option<
        extern "system" fn(
            id: GLuint,
            category: GLenum,
            severity: GLenum,
            length: GLsizei,
            message: *const GLchar,
            user_param: *mut core::ffi::c_void,
        ),
    >;
}


/// How [`GLPointers::load`] (and, by extension, [`GLContext::init`][crate::GLContext::init]) should behave when an
/// OpenGL function pointer fails to load.
///
/// A function pointer "fails to load" if `loader_fn` does not return a non-null pointer after attempting all fallbacks.
///
/// # Safety
///
/// Options other than `Abort` are should be considered `unsafe`. See the documentation on the individual variants for
/// more information.
#[derive(Debug, Clone, Copy)]
pub enum InitFailureMode {
    /// If any one function pointer fails to load, initialization immediately returns an [`Err`].
    ///
    /// As far as Rust safety guarantees go, **this is the only safe option.** All other options will result in a
    /// [`GLPointers`] struct which contains some null pointers (though they are guaranteed to be null, as opposed to be
    /// dangling), which violates Rust's initialization invariant.
    Abort,

    /// For any function pointers that fail to load, a warning is logged using [`log::warn`], and initialization
    /// continues.
    ///
    /// When using this mode, [`GLPointers::load`] and [`GLContext::init`][crate::GLContext::init] are guaranteed to
    /// return [`Ok`].
    ///
    /// Any function pointer that fails to load is guaranteed to be left as [`null`][std::ptr::null] (as opposed to a
    /// dangling pointer) in [`GLPointers`].
    ///
    /// # Safety
    ///
    /// **This option is unsafe.** If any function pointer fails to load, [`GLPointers`] will be left partially
    /// uninitialized, violating Rust's initialization invariant.
    ///
    /// This mode is intended for use during development/debugging to avoid having to gracefully handle all edge-cases.
    /// It should probably not be used in production code.
    WarnAndContinue,

    /// When a function pointer fails to load, initialization continues as if nothing happened. The corresponding
    /// function pointer in [`GLPointers`] will be left as [`null`][std::ptr::null] instead (as opposed to a dangling
    /// pointer).
    ///
    /// # Safety
    ///
    /// **This option is unsafe.** If any function pointer fails to load, [`GLPointers`] will be left partially
    /// uninitialized, violating Rust's initialization invariant.
    ///
    /// This mode is probably unwise to use in production. Consider [`Abort`] and [`WarnAndContinue`] instead.
    ///
    /// [`Abort`]: InitFailureMode::Abort
    /// [`WarnAndContinue`]: InitFailureMode::WarnAndContinue
    ContinueSilently,
}

impl Default for InitFailureMode {
    fn default() -> Self {
        Self::Abort
    }
}


// Include the rest of the bindings, including `GLPointers`, from the build script
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

//! Raw OpenGL bindings that map directly to `extern` function calls.
//!
//! Most of this module's functionality lives within the [`GLPointers`] struct.

use core::ffi::{c_char, c_void};

use types::*;

type VoidPtr = *const c_void;

/// FFI-style type aliases to allow for nicer use of
pub mod types {
    use super::c_char;

    // Standard type aliases
    // -----------------------------------------------------------------------------

    // These types are defined in table 2.2 on page 13 of the 4.6 core specification. They are specifically *not* C
    // types, but have hard-coded sizes. Don't forget that many also have different semantic meanings, despite mapping
    // to the same underlying types.

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
    pub type GLsizei = i32;
    // The OpenGL spec calls theses "non-negative binary integer sizes", so you would expect them to be be u32;
    // however, gl.xml uses a plain `int` instead of `unsigned int` or `uint32_t`. We do the same for consistency.

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

    // Opaque types, used as pointees in some other type definitions.
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


/// How [`GLPointers::init`] should behave when a function pointer cannot be loaded.
///
/// Be careful with using options other than `Abort`. They should be considered `unsafe`. See their documentation for
/// more information.
#[derive(Debug, Clone, Copy)]
pub enum InitFailureMode {
    /// Any single function pointer failing to load will cause [`GLPointers` initialization][GLPointers] to stop and
    /// return an [`Err`].
    ///
    /// As far as Rust safety guarantees go, **this is the only safe option.** All other options will result in
    /// `GLPointers` being partially uninitialized, which violates Rust's initialization invariant.
    Abort,
    /// When a function pointer fails to load, a warning will [be logged][https://docs.rs/log] and initialization will
    /// continue. The function pointer will be left as [`null`][std::ptr::null] instead.
    ///
    /// **This option is unsafe.** If a pointer fails to load, [`GLPointers`] will be left partially uninitialized,
    /// which violates Rust's initialization invariant.
    WarnAndContinue,
    /// When a function pointer fails to load, initialization will continue as if it did not. The function pointer will
    /// be left as [`null`][std::ptr::null] instead.
    ///
    /// **This option is unsafe.** If a pointer fails to load, [`GLPointers`] will be left partially uninitialized,
    /// which violates Rust's initialization invariant.
    ContinueSilently,
}


// Include the rest of the bindings, including `GLPointers`, from the build script
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

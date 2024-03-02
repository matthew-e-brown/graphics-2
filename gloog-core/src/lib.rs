mod funcs;
mod macros;
pub mod raw;
pub mod types;

pub(crate) use crate::macros::*;
use crate::raw::GLPointers;
pub use crate::raw::InitFailureMode;
use crate::types::DebugMessage;


/// A wrapper for an underlying collection of OpenGL functions.
///
/// According to the specification, function pointers loaded for OpenGL are not valid on threads other than the one that
/// loaded them. As such, this type is not [`Send`] or [`Sync`].
pub struct GLContext {
    /// Collection of loaded OpenGL function pointers.
    gl: GLPointers,

    /// The current OpenGL debug callback. Closures stored here need to be [`Sync`] because OpenGL may execute them from
    /// another thread when doing logging. Methods in this crate are **guaranteed** not to call this function.
    debug_callback: Option<Box<dyn FnMut(DebugMessage) + Sync + 'static>>,
}

impl Drop for GLContext {
    fn drop(&mut self) {
        // Ensure that OpenGL doesn't try to call our debugging callback when this struct goes away.
        self.unset_debug_message_callback();
    }
}


// Other implementations are in other files: see the `funcs` module.

impl GLContext {
    /// Initializes the context by loading all `OpenGL` function pointers using the given function to load function
    /// pointers.
    ///
    /// For example, when using with GLFW:
    ///
    /// ```ignore
    /// let gl = GLContext::init(|f| glfw.get_proc_address(f));
    /// ```
    ///
    /// This function returns `Err(&str)` in the event that loading a function fails. The returned string is the
    /// name of the function/symbol that failed to load. A function "fails to load" if the `loader_fn` does not
    /// return a non-null pointer after attempting all fallbacks.
    pub fn init(
        loader_fn: impl FnMut(&'static str) -> *const core::ffi::c_void,
        failure_mode: InitFailureMode,
    ) -> Result<Self, &'static str> {
        let raw_ptrs = unsafe { GLPointers::load(loader_fn, failure_mode) };
        match raw_ptrs {
            Ok(gl) => Ok(Self { gl, debug_callback: None }),
            Err(e) => Err(e),
        }
    }
}


// # Enabling Optimus Support
//
// Should be as simple as:
// https://stackoverflow.com/a/39047129/
// https://stackoverflow.com/a/68471374/
//
// But I couldn't get it to work. Requires exporting some symbols, and no amount of `no_mangle` would get Rust to
// include them. Best I could find is using RUSTFLAGS='-C link-args=-export-dynamic', but that only works if the crate
// is type="dylib". It also made it fail to run on Windows because it tries to link to std dynamically.
//
// Possible next step: maybe could make a C or C++ file with the export declarations in them and `extern` those with
// Rust? Problem then is getting that C file to compile on Windows. Will need to add some stuff to the build script.
//
// Code from first attempt:

// #[cfg(feature = "optimus")]
// #[allow(non_upper_case_globals)]
// #[no_mangle]
// pub static NvOptimusEnablement: u32 = 1;

// #[cfg(feature = "optimus")]
// #[allow(non_upper_case_globals)]
// #[no_mangle]
// // cspell:disable-next-line 'Xpress'
// pub static AmdPowerXpressRequestHighPerformance: i32 = 1;

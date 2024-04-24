mod debug;
mod macros;
pub mod raw;

use debug::DebugMessage;
pub(crate) use macros::*;
use raw::GLPointers;
pub use raw::InitFailureMode;


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


// Further implementation is spread out around the crate.
impl GLContext {
    /// Initializes the context by loading all `OpenGL` function pointers using the given function to load function
    /// pointers.
    ///
    /// For example, when using with GLFW:
    ///
    /// ```ignore
    /// let gl = GLContext::init(|f| glfw.get_proc_address(f), InitFailureMode::Abort);
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

mod funcs;
mod macros;
pub mod types;

pub(crate) use self::macros::*;

/// Raw OpenGL bindings, generated from the specification.
pub mod bindings {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}


/// A wrapper for an underlying collection of OpenGL functions.
///
/// According to the specification, function pointers loaded for OpenGL are not valid on threads other than the one that
/// loaded them. As such, this type is not [`Send`] or [`Sync`].
pub struct GLContext {
    gl: bindings::GLPointers,
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
    pub fn init(loader_fn: impl FnMut(&'static str) -> *const core::ffi::c_void) -> Result<Self, &'static str> {
        Ok(Self {
            gl: bindings::GLPointers::init(loader_fn)?,
        })
    }
}


pub mod new;

use std::rc::Rc;

pub(crate) use self::macros::*;


// ---------------------------------------------------------------


mod funcs;
mod macros;
pub mod types;

/// Raw OpenGL bindings, generated from the specification.
pub mod bindings {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}


// ---------------------------------------------------------------


/// A wrapper for an underlying collection of OpenGL functions.
///
/// According to the specification, function pointers loaded for OpenGL are not valid on threads other than the one that
/// loaded them. As such, this type is not [`Send`] or [`Sync`].
///
/// This struct wraps an [`Rc`], and so it can be cloned fairly cheaply to give function pointers to multiple objects.
#[derive(Clone)]
pub struct GLContext {
    gl: Rc<bindings::GLFunctions>,
}

// Other implementations are in other files: see the `funcs` module.

impl GLContext {
    /// Initializes the context by loading all `OpenGL` function pointers using the given function to load function
    /// pointers.
    ///
    /// For example, when using with GLFW:
    ///
    /// ```ignore
    /// let gl = GLFunctions::init(|f| glfw.get_proc_address(f));
    /// ```
    ///
    /// This function returns `Err(&str)` in the event that loading a function fails. The returned string is the
    /// name of the function/symbol that failed to load. A function "fails to load" if the `loader_fn` does not
    /// return a non-null pointer after attempting all fallbacks.
    pub fn init<F>(loader_fn: F) -> Result<Self, &'static str>
    where
        F: FnMut(&'static str) -> *const core::ffi::c_void,
    {
        Ok(Self {
            gl: Rc::new(bindings::GLFunctions::init(loader_fn)?),
        })
    }
}

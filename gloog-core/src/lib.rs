pub mod debug;
mod macros;
mod meta;
mod objects;
pub mod raw;

use std::cell::Cell;
use std::rc::Rc;

use debug::DebugClosure;
pub(crate) use macros::*;
pub use meta::*;
pub use objects::shader;
use raw::GLPointers;
pub use raw::InitFailureMode;
use shader::Program;


gl_bitfield! {
    /// A bitmask indicating which buffers are to be cleared in [`GLContext::clear`].
    pub struct ClearMask {
        pub const COLOR = COLOR_BUFFER_BIT;
        pub const DEPTH = DEPTH_BUFFER_BIT;
        pub const STENCIL = STENCIL_BUFFER_BIT;
    }
}


/// A wrapper for an underlying collection of OpenGL functions.
///
/// According to the specification, function pointers loaded for OpenGL are not valid on threads other than the one that
/// loaded them. As such, this type is not [`Send`] or [`Sync`].
///
/// # A small note on drop order
///
/// Due to how this struct is implemented, there's a small note worth mentioning about drop order.
///
/// Any OpenGL functions used after this context struct has been dropped will not have their debug messages logged,
/// since `debug_callback` will vanish along with this struct. This type unsets the debug callback when it is dropped,
/// so there is no issue with OpenGL calling a since-dropped function pointer, but that means any previously passed
/// closures will stop working.
///
/// You should always drop this struct **after** any other structs you've created (e.g. [`Program`] objects, [`Shader`]
/// objects, etc.).
///
/// [`Shader`]: shader::Shader
/// [`Program`]: shader::Program
pub struct GLContext {
    /// Collection of loaded OpenGL function pointers.
    gl: Rc<GLPointers>,

    /// The current OpenGL debug callback. Closures stored here need to be [`Sync`] because OpenGL may execute them from
    /// another thread when doing logging. Methods in this crate are **guaranteed** not to call this function.
    debug_callback: Cell<Option<DebugClosure>>,
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
        let raw = unsafe { GLPointers::load(loader_fn, failure_mode) }?;
        Ok(Self {
            gl: Rc::new(raw),
            debug_callback: Cell::new(None),
        })
    }

    /// Make a program the current program object.
    ///
    /// # Panics
    ///
    /// This method panics of `program` has not yet been linked.
    pub fn use_program(&mut self, program: &Program) {
        if !program.is_linked() {
            panic!("attempted to 'use' a non-linked shader program object");
        }

        unsafe { self.gl.use_program(program.id) }
    }

    /// Sets the x, y, width, and height parameters of all viewports.
    pub fn viewport(&mut self, x: i32, y: i32, width: i32, height: i32) {
        unsafe { self.gl.viewport(x, y, width, height) }
    }

    /// Sets every pixel of the relevant buffers to their current clear values.
    pub fn clear(&mut self, buffers: ClearMask) {
        unsafe { self.gl.clear(buffers.into_raw()) }
    }

    /// Sets the clear value for fixed- and floating-point color buffers.
    pub fn clear_color(&mut self, red: f32, green: f32, blue: f32, alpha: f32) {
        unsafe { self.gl.clear_color(red, green, blue, alpha) }
    }

    /// Sets the depth value used when clearing the depth buffer.
    ///
    /// See also [`clear_depth_f`][Self::clear_depth_f].
    pub fn clear_depth(&mut self, depth: f64) {
        unsafe { self.gl.clear_depth(depth) }
    }

    /// Sets the depth value used when clearing the depth buffer.
    ///
    /// See also [`clear_depth`][Self::clear_depth].
    pub fn clear_depth_f(&mut self, depth: f32) {
        unsafe { self.gl.clear_depth_f(depth) }
    }

    /// Sets the value with which the stencil buffer is cleared with. `s` is masked to the number of bitplanes in the
    /// stencil buffer.
    pub fn clear_stencil(&mut self, s: i32) {
        unsafe { self.gl.clear_stencil(s) }
    }
}

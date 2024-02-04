mod buffers;
mod shaders;
mod uniforms;
mod vertex;

use gl::types::*;

pub use self::buffers::*;
pub use self::shaders::*;
pub use self::uniforms::*;
pub use self::vertex::*;
use crate::gl_convert;
use crate::types::{ClearMask, EnableCap};


pub fn load_with<F>(load_fn: F)
where
    F: FnMut(&'static str) -> *const std::ffi::c_void,
{
    gl::load_with(load_fn)
}


pub fn viewport(x: i32, y: i32, width: i32, height: i32) {
    let x = gl_convert!(x, GLint, "viewport x-position");
    let y = gl_convert!(y, GLint, "viewport y-position");
    let width = gl_convert!(width, GLsizei, "viewport width");
    let height = gl_convert!(height, GLsizei, "viewport height");
    unsafe {
        gl::Viewport(x, y, width, height);
    }
}


pub fn clear_color(red: f32, green: f32, blue: f32, alpha: f32) {
    let msg = "clear color should be made of valid `GLfloat`s";
    let r = gl_convert!(red, GLfloat, msg: msg);
    let g = gl_convert!(green, GLfloat, msg: msg);
    let b = gl_convert!(blue, GLfloat, msg: msg);
    let a = gl_convert!(alpha, GLfloat, msg: msg);
    unsafe {
        gl::ClearColor(r, g, b, a);
    }
}


pub fn enable(cap: EnableCap) {
    unsafe {
        gl::Enable(cap.raw());
    }
}

pub fn disable(cap: EnableCap) {
    unsafe {
        gl::Disable(cap.raw());
    }
}


pub fn clear(mask: ClearMask) {
    unsafe {
        gl::Clear(mask.raw());
    }
}

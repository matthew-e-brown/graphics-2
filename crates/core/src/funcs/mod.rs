mod buffers;
mod shaders;
mod vertex;

use gl::types::*;

pub use self::buffers::*;
pub use self::shaders::*;
pub use self::vertex::*;
use crate::types::ClearMask;


pub fn load_with<F>(load_fn: F)
where
    F: FnMut(&'static str) -> *const core::ffi::c_void,
{
    gl::load_with(load_fn)
}


pub fn viewport(x: i32, y: i32, width: i32, height: i32) {
    let x: GLint = x.try_into().expect("viewport x-position should fit into `GLint`");
    let y: GLint = y.try_into().expect("viewport y-position should fit into `GLint`");
    let width: GLsizei = width.try_into().expect("viewport width should fit into `GLsizei`");
    let height: GLsizei = height.try_into().expect("viewport height should fit into `GLsizei`");

    unsafe {
        gl::Viewport(x, y, width, height);
    }
}


pub fn clear_color(red: f32, green: f32, blue: f32, alpha: f32) {
    let msg = "clear color should be made of valid `GLfloat`s";
    let r: GLfloat = red.try_into().expect(msg);
    let g: GLfloat = green.try_into().expect(msg);
    let b: GLfloat = blue.try_into().expect(msg);
    let a: GLfloat = alpha.try_into().expect(msg);

    unsafe {
        gl::ClearColor(r, g, b, a);
    }
}


pub fn clear(mask: ClearMask) {
    unsafe {
        gl::Clear(mask.into());
    }
}

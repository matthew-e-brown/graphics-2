use std::ffi::CString;

use gl::types::*;

use crate::gl_convert;
use crate::types::*;


pub fn get_uniform_location(program: ProgramID, name: &str) -> Option<UniformLocation> {
    let name = CString::new(name).expect("uniform name should not contain nul bytes");
    let loc = unsafe { gl::GetUniformLocation(program.raw(), name.as_ptr()) };
    if loc == -1 {
        None
    } else {
        Some(UniformLocation::new(loc))
    }
}


// https://registry.khronos.org/OpenGL-Refpages/gl4/html/glUniform.xhtml
// TODO: GENERATE THESE WITH A MACRO!!!


pub fn uniform_1f(location: UniformLocation, v0: f32) {
    let v0 = gl_convert!(v0, GLfloat, "uniform value");
    unsafe {
        gl::Uniform1f(location.raw(), v0);
    }
}


pub fn uniform_2f(location: UniformLocation, v0: f32, v1: f32) {
    let v0 = gl_convert!(v0, GLfloat, "uniform value");
    let v1 = gl_convert!(v1, GLfloat, "uniform value");
    unsafe {
        gl::Uniform2f(location.raw(), v0, v1);
    }
}


pub fn uniform_3f(location: UniformLocation, v0: f32, v1: f32, v2: f32) {
    let v0 = gl_convert!(v0, GLfloat, "uniform value");
    let v1 = gl_convert!(v1, GLfloat, "uniform value");
    let v2 = gl_convert!(v2, GLfloat, "uniform value");
    unsafe {
        gl::Uniform3f(location.raw(), v0, v1, v2);
    }
}


pub fn uniform_4f(location: UniformLocation, v0: f32, v1: f32, v2: f32, v3: f32) {
    let v0 = gl_convert!(v0, GLfloat, "uniform value");
    let v1 = gl_convert!(v1, GLfloat, "uniform value");
    let v2 = gl_convert!(v2, GLfloat, "uniform value");
    let v3 = gl_convert!(v3, GLfloat, "uniform value");
    unsafe {
        gl::Uniform4f(location.raw(), v0, v1, v2, v3);
    }
}


pub fn uniform_1i(location: UniformLocation, v0: i32) {
    let v0 = gl_convert!(v0, GLint, "uniform value");
    unsafe {
        gl::Uniform1i(location.raw(), v0);
    }
}


pub fn uniform_2i(location: UniformLocation, v0: i32, v1: i32) {
    let v0 = gl_convert!(v0, GLint, "uniform value");
    let v1 = gl_convert!(v1, GLint, "uniform value");
    unsafe {
        gl::Uniform2i(location.raw(), v0, v1);
    }
}


pub fn uniform_3i(location: UniformLocation, v0: i32, v1: i32, v2: i32) {
    let v0 = gl_convert!(v0, GLint, "uniform value");
    let v1 = gl_convert!(v1, GLint, "uniform value");
    let v2 = gl_convert!(v2, GLint, "uniform value");
    unsafe {
        gl::Uniform3i(location.raw(), v0, v1, v2);
    }
}


pub fn uniform_4i(location: UniformLocation, v0: i32, v1: i32, v2: i32, v3: i32) {
    let v0 = gl_convert!(v0, GLint, "uniform value");
    let v1 = gl_convert!(v1, GLint, "uniform value");
    let v2 = gl_convert!(v2, GLint, "uniform value");
    let v3 = gl_convert!(v3, GLint, "uniform value");
    unsafe {
        gl::Uniform4i(location.raw(), v0, v1, v2, v3);
    }
}


pub fn uniform_1ui(location: UniformLocation, v0: u32) {
    let v0 = gl_convert!(v0, GLuint, "uniform value");
    unsafe {
        gl::Uniform1ui(location.raw(), v0);
    }
}


pub fn uniform_2ui(location: UniformLocation, v0: u32, v1: u32) {
    let v0 = gl_convert!(v0, GLuint, "uniform value");
    let v1 = gl_convert!(v1, GLuint, "uniform value");
    unsafe {
        gl::Uniform2ui(location.raw(), v0, v1);
    }
}


pub fn uniform_3ui(location: UniformLocation, v0: u32, v1: u32, v2: u32) {
    let v0 = gl_convert!(v0, GLuint, "uniform value");
    let v1 = gl_convert!(v1, GLuint, "uniform value");
    let v2 = gl_convert!(v2, GLuint, "uniform value");
    unsafe {
        gl::Uniform3ui(location.raw(), v0, v1, v2);
    }
}


pub fn uniform_4ui(location: UniformLocation, v0: u32, v1: u32, v2: u32, v3: u32) {
    let v0 = gl_convert!(v0, GLuint, "uniform value");
    let v1 = gl_convert!(v1, GLuint, "uniform value");
    let v2 = gl_convert!(v2, GLuint, "uniform value");
    let v3 = gl_convert!(v3, GLuint, "uniform value");
    unsafe {
        gl::Uniform4ui(location.raw(), v0, v1, v2, v3);
    }
}


pub fn uniform_1fv(location: UniformLocation, values: &[[f32; 1]]) {
    let count = gl_convert!(values.len(), GLsizei, "uniform value count");
    unsafe {
        gl::Uniform1fv(location.raw(), count, values.as_ptr().cast());
    }
}


pub fn uniform_2fv(location: UniformLocation, values: &[[f32; 2]]) {
    let count = gl_convert!(values.len(), GLsizei, "uniform value count");
    unsafe {
        gl::Uniform2fv(location.raw(), count, values.as_ptr().cast());
    }
}


pub fn uniform_3fv(location: UniformLocation, values: &[[f32; 3]]) {
    let count = gl_convert!(values.len(), GLsizei, "uniform value count");
    unsafe {
        gl::Uniform3fv(location.raw(), count, values.as_ptr().cast());
    }
}


pub fn uniform_4fv(location: UniformLocation, values: &[[f32; 4]]) {
    let count = gl_convert!(values.len(), GLsizei, "uniform value count");
    unsafe {
        gl::Uniform4fv(location.raw(), count, values.as_ptr().cast());
    }
}


pub fn uniform_1iv(location: UniformLocation, values: &[[i32; 1]]) {
    let count = gl_convert!(values.len(), GLsizei, "uniform value count");
    unsafe {
        gl::Uniform1iv(location.raw(), count, values.as_ptr().cast());
    }
}


pub fn uniform_2iv(location: UniformLocation, values: &[[i32; 2]]) {
    let count = gl_convert!(values.len(), GLsizei, "uniform value count");
    unsafe {
        gl::Uniform2iv(location.raw(), count, values.as_ptr().cast());
    }
}


pub fn uniform_3iv(location: UniformLocation, values: &[[i32; 3]]) {
    let count = gl_convert!(values.len(), GLsizei, "uniform value count");
    unsafe {
        gl::Uniform3iv(location.raw(), count, values.as_ptr().cast());
    }
}


pub fn uniform_4iv(location: UniformLocation, values: &[[i32; 4]]) {
    let count = gl_convert!(values.len(), GLsizei, "uniform value count");
    unsafe {
        gl::Uniform4iv(location.raw(), count, values.as_ptr().cast());
    }
}


pub fn uniform_1uiv(location: UniformLocation, values: &[[u32; 1]]) {
    let count = gl_convert!(values.len(), GLsizei, "uniform value count");
    unsafe {
        gl::Uniform1uiv(location.raw(), count, values.as_ptr().cast());
    }
}


pub fn uniform_2uiv(location: UniformLocation, values: &[[u32; 2]]) {
    let count = gl_convert!(values.len(), GLsizei, "uniform value count");
    unsafe {
        gl::Uniform2uiv(location.raw(), count, values.as_ptr().cast());
    }
}


pub fn uniform_3uiv(location: UniformLocation, values: &[[u32; 3]]) {
    let count = gl_convert!(values.len(), GLsizei, "uniform value count");
    unsafe {
        gl::Uniform3uiv(location.raw(), count, values.as_ptr().cast());
    }
}


pub fn uniform_4uiv(location: UniformLocation, values: &[[u32; 4]]) {
    let count = gl_convert!(values.len(), GLsizei, "uniform value count");
    unsafe {
        gl::Uniform4uiv(location.raw(), count, values.as_ptr().cast());
    }
}


pub fn uniform_matrix_2fv(location: UniformLocation, transpose: bool, values: &[[[f32; 2]; 2]]) {
    let count = gl_convert!(values.len(), GLsizei, "uniform value count");
    unsafe {
        gl::UniformMatrix2fv(location.raw(), count, transpose as GLboolean, values.as_ptr().cast());
    }
}


pub fn uniform_matrix_3fv(location: UniformLocation, transpose: bool, values: &[[[f32; 3]; 3]]) {
    let count = gl_convert!(values.len(), GLsizei, "uniform value count");
    unsafe {
        gl::UniformMatrix3fv(location.raw(), count, transpose as GLboolean, values.as_ptr().cast());
    }
}


pub fn uniform_matrix_4fv(location: UniformLocation, transpose: bool, values: &[[[f32; 4]; 4]]) {
    let count = gl_convert!(values.len(), GLsizei, "uniform value count");
    unsafe {
        gl::UniformMatrix4fv(location.raw(), count, transpose as GLboolean, values.as_ptr().cast());
    }
}


pub fn uniform_matrix_2x3fv(location: UniformLocation, transpose: bool, values: &[[[f32; 3]; 2]]) {
    let count = gl_convert!(values.len(), GLsizei, "uniform value count");
    unsafe {
        gl::UniformMatrix2x3fv(location.raw(), count, transpose as GLboolean, values.as_ptr().cast());
    }
}


pub fn uniform_matrix_3x2fv(location: UniformLocation, transpose: bool, values: &[[[f32; 2]; 3]]) {
    let count = gl_convert!(values.len(), GLsizei, "uniform value count");
    unsafe {
        gl::UniformMatrix3x2fv(location.raw(), count, transpose as GLboolean, values.as_ptr().cast());
    }
}


pub fn uniform_matrix_2x4fv(location: UniformLocation, transpose: bool, values: &[[[f32; 4]; 2]]) {
    let count = gl_convert!(values.len(), GLsizei, "uniform value count");
    unsafe {
        gl::UniformMatrix2x4fv(location.raw(), count, transpose as GLboolean, values.as_ptr().cast());
    }
}


pub fn uniform_matrix_4x2fv(location: UniformLocation, transpose: bool, values: &[[[f32; 2]; 4]]) {
    let count = gl_convert!(values.len(), GLsizei, "uniform value count");
    unsafe {
        gl::UniformMatrix4x2fv(location.raw(), count, transpose as GLboolean, values.as_ptr().cast());
    }
}


pub fn uniform_matrix_3x4fv(location: UniformLocation, transpose: bool, values: &[[[f32; 4]; 3]]) {
    let count = gl_convert!(values.len(), GLsizei, "uniform value count");
    unsafe {
        gl::UniformMatrix3x4fv(location.raw(), count, transpose as GLboolean, values.as_ptr().cast());
    }
}


pub fn uniform_matrix_4x3fv(location: UniformLocation, transpose: bool, values: &[[[f32; 3]; 4]]) {
    let count = gl_convert!(values.len(), GLsizei, "uniform value count");
    unsafe {
        gl::UniformMatrix4x3fv(location.raw(), count, transpose as GLboolean, values.as_ptr().cast());
    }
}

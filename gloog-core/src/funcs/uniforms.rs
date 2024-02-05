use std::ffi::CString;

use crate::bindings::types::*;
use crate::types::*;
use crate::{convert, GLContext};

impl GLContext {
    pub fn get_uniform_location(&self, program: ProgramID, name: &str) -> Option<UniformLocation> {
        let name = CString::new(name).expect("uniform name should not contain NUL-bytes");
        let loc = unsafe { self.gl.get_uniform_location(program.into_raw(), name.as_ptr()) };
        if loc != -1 {
            Some(UniformLocation::new(loc))
        } else {
            None
        }
    }

    // https://registry.khronos.org/OpenGL-Refpages/gl4/html/glUniform.xhtml
    // TODO: GENERATE THESE WITH A MACRO!!!

    pub fn uniform_1f(&self, location: UniformLocation, v0: f32) {
        let v0 = convert!(v0, GLfloat, "uniform value");
        unsafe { self.gl.uniform_1f(location.into_raw(), v0) }
    }

    pub fn uniform_2f(&self, location: UniformLocation, v0: f32, v1: f32) {
        let v0 = convert!(v0, GLfloat, "uniform value");
        let v1 = convert!(v1, GLfloat, "uniform value");
        unsafe { self.gl.uniform_2f(location.into_raw(), v0, v1) }
    }

    pub fn uniform_3f(&self, location: UniformLocation, v0: f32, v1: f32, v2: f32) {
        let v0 = convert!(v0, GLfloat, "uniform value");
        let v1 = convert!(v1, GLfloat, "uniform value");
        let v2 = convert!(v2, GLfloat, "uniform value");
        unsafe { self.gl.uniform_3f(location.into_raw(), v0, v1, v2) }
    }

    pub fn uniform_4f(&self, location: UniformLocation, v0: f32, v1: f32, v2: f32, v3: f32) {
        let v0 = convert!(v0, GLfloat, "uniform value");
        let v1 = convert!(v1, GLfloat, "uniform value");
        let v2 = convert!(v2, GLfloat, "uniform value");
        let v3 = convert!(v3, GLfloat, "uniform value");
        unsafe { self.gl.uniform_4f(location.into_raw(), v0, v1, v2, v3) }
    }

    pub fn uniform_1i(&self, location: UniformLocation, v0: i32) {
        let v0 = convert!(v0, GLint, "uniform value");
        unsafe { self.gl.uniform_1i(location.into_raw(), v0) }
    }

    pub fn uniform_2i(&self, location: UniformLocation, v0: i32, v1: i32) {
        let v0 = convert!(v0, GLint, "uniform value");
        let v1 = convert!(v1, GLint, "uniform value");
        unsafe { self.gl.uniform_2i(location.into_raw(), v0, v1) }
    }

    pub fn uniform_3i(&self, location: UniformLocation, v0: i32, v1: i32, v2: i32) {
        let v0 = convert!(v0, GLint, "uniform value");
        let v1 = convert!(v1, GLint, "uniform value");
        let v2 = convert!(v2, GLint, "uniform value");
        unsafe { self.gl.uniform_3i(location.into_raw(), v0, v1, v2) }
    }

    pub fn uniform_4i(&self, location: UniformLocation, v0: i32, v1: i32, v2: i32, v3: i32) {
        let v0 = convert!(v0, GLint, "uniform value");
        let v1 = convert!(v1, GLint, "uniform value");
        let v2 = convert!(v2, GLint, "uniform value");
        let v3 = convert!(v3, GLint, "uniform value");
        unsafe { self.gl.uniform_4i(location.into_raw(), v0, v1, v2, v3) }
    }

    pub fn uniform_1ui(&self, location: UniformLocation, v0: u32) {
        let v0 = convert!(v0, GLuint, "uniform value");
        unsafe { self.gl.uniform_1ui(location.into_raw(), v0) }
    }

    pub fn uniform_2ui(&self, location: UniformLocation, v0: u32, v1: u32) {
        let v0 = convert!(v0, GLuint, "uniform value");
        let v1 = convert!(v1, GLuint, "uniform value");
        unsafe { self.gl.uniform_2ui(location.into_raw(), v0, v1) }
    }

    pub fn uniform_3ui(&self, location: UniformLocation, v0: u32, v1: u32, v2: u32) {
        let v0 = convert!(v0, GLuint, "uniform value");
        let v1 = convert!(v1, GLuint, "uniform value");
        let v2 = convert!(v2, GLuint, "uniform value");
        unsafe { self.gl.uniform_3ui(location.into_raw(), v0, v1, v2) }
    }

    pub fn uniform_4ui(&self, location: UniformLocation, v0: u32, v1: u32, v2: u32, v3: u32) {
        let v0 = convert!(v0, GLuint, "uniform value");
        let v1 = convert!(v1, GLuint, "uniform value");
        let v2 = convert!(v2, GLuint, "uniform value");
        let v3 = convert!(v3, GLuint, "uniform value");
        unsafe { self.gl.uniform_4ui(location.into_raw(), v0, v1, v2, v3) }
    }

    pub fn uniform_1fv(&self, location: UniformLocation, values: &[[f32; 1]]) {
        let count = convert!(values.len(), GLsizei, "uniform value count");
        unsafe { self.gl.uniform_1fv(location.into_raw(), count, values.as_ptr().cast()) }
    }

    pub fn uniform_2fv(&self, location: UniformLocation, values: &[[f32; 2]]) {
        let count = convert!(values.len(), GLsizei, "uniform value count");
        unsafe { self.gl.uniform_2fv(location.into_raw(), count, values.as_ptr().cast()) }
    }

    pub fn uniform_3fv(&self, location: UniformLocation, values: &[[f32; 3]]) {
        let count = convert!(values.len(), GLsizei, "uniform value count");
        unsafe { self.gl.uniform_3fv(location.into_raw(), count, values.as_ptr().cast()) }
    }

    pub fn uniform_4fv(&self, location: UniformLocation, values: &[[f32; 4]]) {
        let count = convert!(values.len(), GLsizei, "uniform value count");
        unsafe { self.gl.uniform_4fv(location.into_raw(), count, values.as_ptr().cast()) }
    }

    pub fn uniform_1iv(&self, location: UniformLocation, values: &[[i32; 1]]) {
        let count = convert!(values.len(), GLsizei, "uniform value count");
        unsafe { self.gl.uniform_1iv(location.into_raw(), count, values.as_ptr().cast()) }
    }

    pub fn uniform_2iv(&self, location: UniformLocation, values: &[[i32; 2]]) {
        let count = convert!(values.len(), GLsizei, "uniform value count");
        unsafe { self.gl.uniform_2iv(location.into_raw(), count, values.as_ptr().cast()) }
    }

    pub fn uniform_3iv(&self, location: UniformLocation, values: &[[i32; 3]]) {
        let count = convert!(values.len(), GLsizei, "uniform value count");
        unsafe { self.gl.uniform_3iv(location.into_raw(), count, values.as_ptr().cast()) }
    }

    pub fn uniform_4iv(&self, location: UniformLocation, values: &[[i32; 4]]) {
        let count = convert!(values.len(), GLsizei, "uniform value count");
        unsafe { self.gl.uniform_4iv(location.into_raw(), count, values.as_ptr().cast()) }
    }

    pub fn uniform_1uiv(&self, location: UniformLocation, values: &[[u32; 1]]) {
        let count = convert!(values.len(), GLsizei, "uniform value count");
        unsafe { self.gl.uniform_1uiv(location.into_raw(), count, values.as_ptr().cast()) }
    }

    pub fn uniform_2uiv(&self, location: UniformLocation, values: &[[u32; 2]]) {
        let count = convert!(values.len(), GLsizei, "uniform value count");
        unsafe { self.gl.uniform_2uiv(location.into_raw(), count, values.as_ptr().cast()) }
    }

    pub fn uniform_3uiv(&self, location: UniformLocation, values: &[[u32; 3]]) {
        let count = convert!(values.len(), GLsizei, "uniform value count");
        unsafe { self.gl.uniform_3uiv(location.into_raw(), count, values.as_ptr().cast()) }
    }

    pub fn uniform_4uiv(&self, location: UniformLocation, values: &[[u32; 4]]) {
        let count = convert!(values.len(), GLsizei, "uniform value count");
        unsafe { self.gl.uniform_4uiv(location.into_raw(), count, values.as_ptr().cast()) }
    }

    pub fn uniform_matrix_2fv(&self, location: UniformLocation, transpose: bool, values: &[[[f32; 2]; 2]]) {
        let count = convert!(values.len(), GLsizei, "uniform value count");
        let transpose = convert!(transpose, GLboolean, "'transpose' boolean");
        let values = values.as_ptr().cast();
        unsafe { self.gl.uniform_matrix_2fv(location.into_raw(), count, transpose, values) }
    }

    pub fn uniform_matrix_3fv(&self, location: UniformLocation, transpose: bool, values: &[[[f32; 3]; 3]]) {
        let count = convert!(values.len(), GLsizei, "uniform value count");
        let transpose = convert!(transpose, GLboolean, "'transpose' boolean");
        let values = values.as_ptr().cast();
        unsafe { self.gl.uniform_matrix_3fv(location.into_raw(), count, transpose, values) }
    }

    pub fn uniform_matrix_4fv(&self, location: UniformLocation, transpose: bool, values: &[[[f32; 4]; 4]]) {
        let count = convert!(values.len(), GLsizei, "uniform value count");
        let transpose = convert!(transpose, GLboolean, "'transpose' boolean");
        let values = values.as_ptr().cast();
        unsafe { self.gl.uniform_matrix_4fv(location.into_raw(), count, transpose, values) }
    }

    pub fn uniform_matrix_2x3fv(&self, location: UniformLocation, transpose: bool, values: &[[[f32; 3]; 2]]) {
        let count = convert!(values.len(), GLsizei, "uniform value count");
        let transpose = convert!(transpose, GLboolean, "'transpose' boolean");
        let values = values.as_ptr().cast();
        unsafe { self.gl.uniform_matrix_2x3fv(location.into_raw(), count, transpose, values) }
    }

    pub fn uniform_matrix_3x2fv(&self, location: UniformLocation, transpose: bool, values: &[[[f32; 2]; 3]]) {
        let count = convert!(values.len(), GLsizei, "uniform value count");
        let transpose = convert!(transpose, GLboolean, "'transpose' boolean");
        let values = values.as_ptr().cast();
        unsafe { self.gl.uniform_matrix_3x2fv(location.into_raw(), count, transpose, values) }
    }

    pub fn uniform_matrix_2x4fv(&self, location: UniformLocation, transpose: bool, values: &[[[f32; 4]; 2]]) {
        let count = convert!(values.len(), GLsizei, "uniform value count");
        let transpose = convert!(transpose, GLboolean, "'transpose' boolean");
        let values = values.as_ptr().cast();
        unsafe { self.gl.uniform_matrix_2x4fv(location.into_raw(), count, transpose, values) }
    }

    pub fn uniform_matrix_4x2fv(&self, location: UniformLocation, transpose: bool, values: &[[[f32; 2]; 4]]) {
        let count = convert!(values.len(), GLsizei, "uniform value count");
        let transpose = convert!(transpose, GLboolean, "'transpose' boolean");
        let values = values.as_ptr().cast();
        unsafe { self.gl.uniform_matrix_4x2fv(location.into_raw(), count, transpose, values) }
    }

    pub fn uniform_matrix_3x4fv(&self, location: UniformLocation, transpose: bool, values: &[[[f32; 4]; 3]]) {
        let count = convert!(values.len(), GLsizei, "uniform value count");
        let transpose = convert!(transpose, GLboolean, "'transpose' boolean");
        let values = values.as_ptr().cast();
        unsafe { self.gl.uniform_matrix_3x4fv(location.into_raw(), count, transpose, values) }
    }

    pub fn uniform_matrix_4x3fv(&self, location: UniformLocation, transpose: bool, values: &[[[f32; 3]; 4]]) {
        let count = convert!(values.len(), GLsizei, "uniform value count");
        let transpose = convert!(transpose, GLboolean, "'transpose' boolean");
        let values = values.as_ptr().cast();
        unsafe { self.gl.uniform_matrix_4x3fv(location.into_raw(), count, transpose, values) }
    }
}

use std::ffi::CString;

use crate::raw::types::*;
use crate::types::*;
use crate::{convert, GLContext};

impl GLContext {
    pub fn create_vertex_array(&self) -> VertexArrayID {
        let mut name = 0;
        unsafe { self.gl.create_vertex_arrays(1, &mut name) };
        VertexArrayID::new(name)
    }


    pub fn create_vertex_arrays(&self, n: usize) -> Vec<VertexArrayID> {
        if n == 0 {
            return vec![];
        }

        let mut names = vec![0; n];
        let n = convert!(n, GLsizei, "number of vertex arrays");

        unsafe { self.gl.create_vertex_arrays(n, names.as_mut_ptr()) };
        names.into_iter().map(VertexArrayID::new).collect()
    }


    pub fn bind_vertex_array(&self, vao: VertexArrayID) {
        unsafe { self.gl.bind_vertex_array(vao.into_raw()) }
    }


    pub fn unbind_vertex_array(&self) {
        unsafe { self.gl.bind_vertex_array(0) }
    }


    pub fn get_attrib_location(&self, program: ProgramID, name: &str) -> Option<VertexAttribLocation> {
        let name = CString::new(name).expect("attrib location name should not contain NUL-bytes");
        let loc = unsafe { self.gl.get_attrib_location(program.into_raw(), name.as_ptr()) };
        if loc != -1 {
            Some(VertexAttribLocation(loc as GLuint))
        } else {
            None
        }
    }


    pub fn vertex_attrib_pointer(
        &self,
        index: impl Into<VertexAttribLocation>,
        size: usize,
        attrib_type: VertexAttribType,
        normalized: bool,
        stride: isize,
        offset: usize,
    ) {
        let index = index.into().0;
        let stride = convert!(stride, GLsizei, "vertex attribute stride");
        let normalized = convert!(normalized, GLboolean, "'normalized' parameter");

        let offset = offset as *const _;
        let attrib = attrib_type.into_raw();

        let size = match size {
            n @ 1..=4 => n as GLsizei,
            n if n == (crate::raw::BGRA as usize) => n as GLsizei,
            _ => panic!("vertex attribute size should be 1, 2, 3, 4, or GL_BGRA"),
        };

        unsafe { self.gl.vertex_attrib_pointer(index, size, attrib, normalized, stride, offset) }
    }


    pub fn enable_vertex_attrib_array(&self, index: impl Into<VertexAttribLocation>) {
        let index = index.into().0;
        unsafe { self.gl.enable_vertex_attrib_array(index) }
    }


    pub fn draw_arrays(&self, mode: DrawMode, first: usize, count: usize) {
        let first = convert!(first, GLint, "draw arrays index");
        let count = convert!(count, GLsizei, "draw arrays count");
        unsafe { self.gl.draw_arrays(mode.into_raw(), first, count) }
    }

    pub fn draw_elements(&self, mode: DrawMode, count: usize, ty: DrawElementsType, offset: usize) {
        let count = convert!(count, GLsizei, "draw elements count");
        let indices = offset as *const _;
        unsafe { self.gl.draw_elements(mode.into_raw(), count, ty.into_raw(), indices) }
    }
}

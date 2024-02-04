use crate::types::*;
use crate::{convert, GLContext};

impl GLContext {
    pub fn create_vertex_array(&self) -> VertexArrayID {
        let mut name = 0;
        unsafe { self.funcs.create_vertex_arrays(1, &mut name) };
        VertexArrayID::new(name)
    }

    pub fn create_vertex_arrays(&self, n: usize) -> Vec<VertexArrayID> {
        if n == 0 {
            return vec![];
        }

        let mut names = vec![0; n];
        let n = convert!(n, i32, "number of vertex arrays");

        unsafe { self.funcs.create_vertex_arrays(n, names.as_mut_ptr()) };
        names.into_iter().map(VertexArrayID::new).collect()
    }

    pub fn bind_vertex_array(&self, vao: VertexArrayID) {
        unsafe { self.funcs.bind_vertex_array(vao.into()) }
    }

    pub fn unbind_vertex_array(&self) {
        unsafe { self.funcs.bind_vertex_array(0) }
    }

    pub fn vertex_attrib_pointer(
        &self,
        index: u32,
        size: usize,
        attrib_type: VertexAttribType,
        normalized: bool,
        stride: isize,
        offset: usize,
    ) {
        let stride = convert!(stride, i32, "vertex attribute stride");

        let offset = offset as *const _;
        let attrib = attrib_type.into();

        let size = match size {
            n @ 1..=4 => n as i32,
            n if n == crate::raw::BGRA.into_raw() as usize => n as i32,
            _ => panic!("vertex attribute size should be 1, 2, 3, 4, or GL_BGRA"),
        };

        unsafe {
            self.funcs
                .vertex_attrib_pointer(index, size, attrib, normalized, stride, offset)
        }
    }

    pub fn enable_vertex_attrib_array(&self, index: u32) {
        unsafe { self.funcs.enable_vertex_attrib_array(index) }
    }

    pub fn draw_arrays(&self, mode: DrawMode, first: usize, count: usize) {
        let first = convert!(first, i32, "draw arrays index");
        let count = convert!(count, i32, "draw arrays count");
        unsafe { self.funcs.draw_arrays(mode.into(), first, count) }
    }
}

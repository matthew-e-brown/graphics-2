use gl::types::*;

use crate::types::*;


pub fn create_vertex_array() -> VertexArrayID {
    let mut name = 0;
    unsafe {
        gl::CreateVertexArrays(1, &mut name);
    }

    VertexArrayID::new(name)
}


pub fn create_vertex_arrays(n: usize) -> Vec<VertexArrayID> {
    if n == 0 {
        return vec![];
    }

    let mut names = vec![0; n];
    let n: GLsizei = n.try_into().expect("vertex array creation count should fit into `GLsizei`");

    unsafe {
        gl::CreateVertexArrays(n, names.as_mut_ptr());
    }

    names.into_iter().map(VertexArrayID::new).collect()
}


pub fn bind_vertex_array(vao: VertexArrayID) {
    unsafe {
        gl::BindVertexArray(vao.raw());
    }
}


pub fn vertex_attrib_pointer(
    index: usize,
    size: usize,
    attrib_type: VertexAttribType,
    normalized: bool,
    stride: isize,
    offset: usize,
) {
    // todo: check these against gl::MAX_ATTRIB_STRIDE etc.; maybe return GLError enum?
    let index: GLuint = index.try_into().expect("vertex attribute index should fit into `GLuint`");
    let size: GLint = match size {
        n @ 1..=4 => n.try_into().unwrap(),
        _ => panic!("vertex attributes should be between 1-4 components in size"),
    };
    let normalized: GLboolean = normalized.into();
    let stride: GLsizei = stride.try_into().expect("vertex attribute stride should fit into `GLsizei`");
    unsafe {
        gl::VertexAttribPointer(index, size, attrib_type.raw(), normalized, stride, offset as *const _);
    }
}


pub fn enable_vertex_attrib_array(index: usize) {
    let index: GLuint = index.try_into().expect("vertex attribute index should fit into `GLuint`");
    unsafe {
        gl::EnableVertexAttribArray(index);
    }
}


pub fn draw_arrays(mode: DrawMode, first: usize, count: usize) {
    let first: GLint = first.try_into().expect("array index to draw should fit into `GLint`");
    let count: GLsizei = count.try_into().expect("array count to draw should fit into `GLsizei`");
    unsafe {
        gl::DrawArrays(mode.raw(), first, count);
    }
}

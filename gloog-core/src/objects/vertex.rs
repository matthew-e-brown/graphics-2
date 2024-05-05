use std::mem::ManuallyDrop;
use std::ptr;
use std::rc::Rc;

use crate::raw::types::*;
use crate::raw::GLPointers;
use crate::{convert, GLContext};


impl GLContext {
    pub fn create_vertex_array(&self) -> VertexArray {
        let ptrs = Rc::clone(&self.gl);
        let mut name: GLuint = 0;

        unsafe {
            self.gl.create_vertex_arrays(1, &mut name);
        };

        VertexArray { gl: ptrs, id: name }
    }

    pub fn create_vertex_arrays(&self, n: usize) -> Vec<VertexArray> {
        let mut names: Vec<GLuint> = vec![0; n];
        let num = convert!(names.len(), GLsizei, "number of vertex array objects");
        unsafe {
            self.gl.create_vertex_arrays(num, names.as_mut_ptr());
        }

        names
            .into_iter()
            .map(|name| VertexArray {
                gl: Rc::clone(&self.gl),
                id: name,
            })
            .collect()
    }

    pub fn bind_vertex_array(&mut self, array: Option<&VertexArray>) {
        unsafe { self.gl.bind_vertex_array(array.map(|arr| arr.id).unwrap_or(0)) }
    }
}


/// An OpenGL vertex array object (VAO).
///
/// Dropping this type calls `glDeleteVertexArrays` (with an array of size 1).
pub struct VertexArray {
    /// A pointer to loaded OpenGL functions.
    pub(crate) gl: Rc<GLPointers>,
    /// The "name" of this vertex array object.
    pub(crate) id: GLuint,
}

impl VertexArray {
    pub const fn id(&self) -> GLuint {
        self.id
    }

    /// Deletes one or more vertex arrays.
    ///
    /// Normally, a VAO can be deleted simply by dropping it. This method allows one to drop many at once.
    pub fn delete<I: IntoIterator<Item = VertexArray>>(arrays: I) {
        let mut local_gl = None; // Grab a handle to at least one `Rc<GLPointers>` as we pluck IDs out of the VAOs
        let names: Vec<GLuint> = arrays
            .into_iter()
            .map(|array| {
                // See Buffer::delete in buffer.rs
                let array = ManuallyDrop::new(array);
                local_gl = Some(unsafe { ptr::read(&array.gl) }); // Store at least one of them; drop old ones.
                array.id
            })
            .collect();

        // If `None`, we have nothing to delete.
        if let Some(gl) = local_gl {
            let len = convert!(names.len(), GLsizei, "number of vertex array objects");
            unsafe { gl.delete_vertex_arrays(len, names.as_ptr()) }
        }
    }

    // [TODO] These methods are going to depend heavily on how buffers and vertex attributes are implemented. Do those
    // first.

    // /// Binds a buffer object to this vertex array object's element array buffer bind point.
    // pub fn element_buffer(&mut self, buffer: ) {
    //     todo!();
    // }

    // pub fn vertex_buffer(&mut self, index: GLuint, buffer: ) {
    //     todo!();
    // }
}

impl Drop for VertexArray {
    fn drop(&mut self) {
        unsafe { self.gl.delete_vertex_arrays(1, &self.id) }
    }
}

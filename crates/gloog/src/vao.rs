use gl::types::*;

use crate::gl_enum;


gl_enum! {
    /// The data type to be used for each component in a vertex attribute's array.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub enum VertexAttrType {
        Byte => BYTE,
        UnsignedByte => UNSIGNED_BYTE,
        Short => SHORT,
        UnsignedShort => UNSIGNED_SHORT,
        Int => INT,
        UnsignedInt => UNSIGNED_INT,
        HalfFloat => HALF_FLOAT,
        Float => FLOAT,
        Double => DOUBLE,
        Fixed => FIXED,
        Int2_10_10_10Rev => INT_2_10_10_10_REV,
        UnsignedInt2_10_10_10Rev => UNSIGNED_INT_2_10_10_10_REV,
    }
}


/// A vertex array object.
#[derive(Debug)]
pub struct VertexArray {
    pub(crate) name: GLuint,
}

impl VertexArray {
    pub fn new() -> Self {
        let mut name = 0;

        unsafe {
            gl::CreateVertexArrays(1, &mut name);
        }

        Self { name }
    }

    pub fn new_multiple(n: usize) -> Vec<Self> {
        assert!(n > 0, "cannot create zero vertex array objects");

        let mut names = vec![0; n];
        let n: GLsizei = n.try_into().expect("VAO creation count should fit into `GLsizei`");

        unsafe {
            gl::CreateVertexArrays(n, names.as_mut_ptr());
        }

        names.into_iter().map(|name| Self { name }).collect()
    }

    pub fn bind(&self) {
        // SAFETY: https://registry.khronos.org/OpenGL-Refpages/gl4/html/glBindVertexArray.xhtml -- only error case is
        // if `name` is not 0 or a previously generated name
        unsafe {
            gl::BindVertexArray(self.name);
        }
    }
}


// TODO: impl Bindable for VertexArray

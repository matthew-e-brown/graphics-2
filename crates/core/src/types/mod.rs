mod enums;
mod flags;
mod macros;

use gl::types::*;

pub use self::enums::*;
pub use self::flags::*;


#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct BufferID(GLuint);

impl BufferID {
    pub(crate) fn new(name: GLuint) -> Self {
        Self(name)
    }

    pub(crate) fn name(&self) -> GLuint {
        self.0
    }
}


#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ShaderID(GLuint, ShaderType);

impl ShaderID {
    pub(crate) fn new(name: GLuint, shader_type: ShaderType) -> Self {
        Self(name, shader_type)
    }

    pub(crate) fn name(&self) -> GLuint {
        self.0
    }

    pub fn shader_type(&self) -> ShaderType {
        self.1
    }
}


#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ProgramID(GLuint);

impl ProgramID {
    pub(crate) fn new(name: GLuint) -> Self {
        Self(name)
    }

    pub(crate) fn name(&self) -> GLuint {
        self.0
    }
}


#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct VertexArrayID(GLuint);

impl VertexArrayID {
    pub(crate) fn new(name: GLuint) -> Self {
        Self(name)
    }

    pub(crate) fn name(&self) -> GLuint {
        self.0
    }
}

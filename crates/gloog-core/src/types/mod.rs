mod enums;
mod flags;
mod macros;

use gl::types::*;

pub use self::enums::*;
pub use self::flags::*;


#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Buffer(GLuint);

impl Buffer {
    pub(crate) const fn new(name: GLuint) -> Self {
        Self(name)
    }

    pub(crate) const fn name(&self) -> GLuint {
        self.0
    }
}


#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Shader(GLuint, ShaderType);

impl Shader {
    pub(crate) const fn new(name: GLuint, shader_type: ShaderType) -> Self {
        Self(name, shader_type)
    }

    pub(crate) const fn name(&self) -> GLuint {
        self.0
    }

    pub const fn shader_type(&self) -> ShaderType {
        self.1
    }
}


#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Program(GLuint);

impl Program {
    pub(crate) const fn new(name: GLuint) -> Self {
        Self(name)
    }

    pub(crate) const fn name(&self) -> GLuint {
        self.0
    }
}


#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct VertexArray(GLuint);

impl VertexArray {
    pub(crate) const fn new(name: GLuint) -> Self {
        Self(name)
    }

    pub(crate) const fn name(&self) -> GLuint {
        self.0
    }
}

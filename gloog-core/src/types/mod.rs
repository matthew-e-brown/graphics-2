mod enums;
mod flags;

pub use self::enums::*;
pub use self::flags::*;
use crate::bindings::types::*;
use crate::macros::gl_newtype;

gl_newtype!(pub struct BufferID(GLuint));
gl_newtype!(pub struct ShaderID(GLuint));
gl_newtype!(pub struct ProgramID(GLuint));
gl_newtype!(pub struct VertexArrayID(GLuint));
gl_newtype!(pub struct UniformLocation(pub GLint));
gl_newtype!(pub struct VertexAttribLocation(pub GLuint));

impl Into<VertexAttribLocation> for GLuint {
    fn into(self) -> VertexAttribLocation {
        VertexAttribLocation(self)
    }
}

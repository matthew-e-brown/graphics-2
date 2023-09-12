mod enums;
mod flags;
mod macros;

use gl::types::*;

pub use self::enums::*;
pub use self::flags::*;
use self::macros::gl_wrapper;


gl_wrapper!(pub struct BufferID(GLuint));


gl_wrapper!(pub struct ShaderID(GLuint));


gl_wrapper!(pub struct ProgramID(GLuint));


gl_wrapper!(pub struct VertexArrayID(GLuint));


gl_wrapper!(pub struct UniformLocation(GLint));

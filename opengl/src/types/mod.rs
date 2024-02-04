mod enums;
mod flags;
mod macros;

pub use self::enums::*;
pub use self::flags::*;
use self::macros::gl_newtype;


gl_newtype!(pub struct BufferID(u32));
gl_newtype!(pub struct ShaderID(u32));
gl_newtype!(pub struct ProgramID(u32));
gl_newtype!(pub struct VertexArrayID(u32));
gl_newtype!(pub struct UniformLocation(i32));

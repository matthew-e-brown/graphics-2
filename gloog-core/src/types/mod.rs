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


impl UniformLocation {
    /// Check if this uniform was found in the shader.
    pub const fn is_some(&self) -> bool {
        self.0 != -1
    }
}

impl Into<VertexAttribLocation> for GLuint {
    fn into(self) -> VertexAttribLocation {
        VertexAttribLocation(self)
    }
}


#[derive(Debug, Clone)]
pub struct DebugMessage {
    pub id: u32,
    pub typ: DebugType,
    pub source: DebugSource,
    pub severity: DebugSeverity,
    pub body: String,
}

impl DebugMessage {
    pub fn as_str(&self) -> &str {
        &self.body[..]
    }
}

impl AsRef<str> for DebugMessage {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

#[derive(Debug, Clone, Copy)]
pub enum DebugFilter<'a> {
    /// Enable or disable all messages whose source, type, and severity all match the provided values; a value of `None`
    /// corresponds to `GL_DONT_CARE`.
    Where {
        source: Option<DebugSource>,
        typ: Option<DebugType>,
        severity: Option<DebugSeverity>,
    },
    /// Enable or disable all messages whose source, type, *and* ID values exactly match those provided.
    ById {
        source: DebugSource,
        typ: DebugType,
        ids: &'a [GLuint],
    },
}

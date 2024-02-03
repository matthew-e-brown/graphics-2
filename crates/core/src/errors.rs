use std::fmt;

use thiserror::Error;

/// This error is returned when creating an OpenGL "_object_" fails.
///
/// This error corresponds to when an object-creation function like [`glCreateShader`] or [`glCreateProgram`] returns a
/// value of zero.  There are no extra details associated with it.
///
/// [`glCreateShader`]: https://registry.khronos.org/OpenGL-Refpages/gl4/html/glCreateShader.xhtml
/// [`glCreateProgram`]: https://registry.khronos.org/OpenGL-Refpages/gl4/html/glCreateProgram.xhtml
#[derive(Error, Debug, Clone, Copy)]
#[error("OpenGL failed to create {0}.")] // note: {0} should *include* article ("an ..." object)
pub struct ObjectCreationError(ObjectCreationErrorKind);

impl ObjectCreationError {
    pub(crate) const fn new(kind: ObjectCreationErrorKind) -> Self {
        Self(kind)
    }
}

/// Which object [failed to be created][ObjectCreationError].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObjectCreationErrorKind {
    Shader,
    Program,
}

impl fmt::Display for ObjectCreationErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(match self {
            Self::Shader => "a shader object",
            Self::Program => "a program object",
        })
    }
}

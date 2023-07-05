use bytemuck::{bytes_of, Pod};
use gl::types::*;
pub use {gl, glfw};

pub mod buffers;
pub mod shaders;


/// An error from OpenGL.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OpenGLError {
    /// An unacceptable value is specified for an enumerated argument.
    ///
    /// The offending command is ignored and has no other side effect than to set the error flag.
    InvalidEnum,
    /// A numeric argument is out of range.
    ///
    /// The offending command is ignored and has no other side effect than to set the error flag.
    InvalidValue,
    /// The specified operation is not allowed in the current state.
    ///
    /// The offending command is ignored and has no other side effect than to set the error flag.
    InvalidOperation,
    /// The framebuffer object is not complete.
    ///
    /// The offending command is ignored and has no other side effect than to set the error flag.
    InvalidFramebufferOperation,
    /// There is not enough memory left to execute the command.
    ///
    /// The state of the GL is undefined, except for the state of the error flags, after this error is recorded.
    OutOfMemory,
    /// An attempt has been made to perform an operation that would cause an internal stack to underflow.
    StackUnderflow,
    /// An attempt has been made to perform an operation that would cause an internal stack to overflow.
    StackOverflow,
}

impl OpenGLError {
    /// Returns error information from the OpenGL context, if any.
    ///
    /// This function maps to [`glGetError`].
    ///
    /// [`glGetError`]: https://registry.khronos.org/OpenGL-Refpages/gl4/html/glGetError.xhtml
    pub fn get_error() -> Option<Self> {
        let err = unsafe { gl::GetError() };
        if err == gl::NO_ERROR {
            None
        } else {
            Some(err.into())
        }
    }

    /// Returns error information from the OpenGL context; if there isn't any, return the given value as [`Ok`].
    ///
    /// See [`get_error`][Self::get_error] for more information.
    pub fn get_error_or_ok<T>(default: T) -> Result<T, Self> {
        Self::get_error().map(|e| Err(e)).unwrap_or(Ok(default))
    }

    /// Returns error information from the OpenGL context; if there isn't any, return the return value of the given
    /// function as [`Ok`].
    ///
    /// See [`get_error`][Self::get_error] for more information.
    pub fn get_error_or_else_ok<T>(f: impl FnOnce() -> T) -> Result<T, Self> {
        Self::get_error().map(|e| Err(e)).unwrap_or_else(|| Ok(f()))
    }
}

impl std::error::Error for OpenGLError {}

impl<T> From<OpenGLError> for Result<T, OpenGLError> {
    fn from(value: OpenGLError) -> Self {
        Err(value)
    }
}

impl std::fmt::Display for OpenGLError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str(match self {
            OpenGLError::InvalidEnum => "an unacceptable value is specified for an enumerated argument",
            OpenGLError::InvalidValue => "a numeric argument is out of range",
            OpenGLError::InvalidOperation => "the specified operation is not allowed in the current state",
            OpenGLError::InvalidFramebufferOperation => "the framebuffer object is not complete",
            OpenGLError::OutOfMemory => "there is not enough memory left to execute the command",
            OpenGLError::StackUnderflow => {
                "an attempt has been made to perform an operation that would cause an internal stack to underflow"
            },
            OpenGLError::StackOverflow => {
                "an attempt has been made to perform an operation that would cause an internal stack to overflow"
            },
        })
    }
}

impl From<OpenGLError> for GLenum {
    fn from(value: OpenGLError) -> Self {
        match value {
            OpenGLError::InvalidEnum => gl::INVALID_ENUM,
            OpenGLError::InvalidValue => gl::INVALID_VALUE,
            OpenGLError::InvalidOperation => gl::INVALID_OPERATION,
            OpenGLError::InvalidFramebufferOperation => gl::INVALID_FRAMEBUFFER_OPERATION,
            OpenGLError::OutOfMemory => gl::OUT_OF_MEMORY,
            OpenGLError::StackUnderflow => gl::STACK_UNDERFLOW,
            OpenGLError::StackOverflow => gl::STACK_OVERFLOW,
        }
    }
}

impl From<GLenum> for OpenGLError {
    fn from(value: GLenum) -> Self {
        match value {
            gl::INVALID_ENUM => OpenGLError::InvalidEnum,
            gl::INVALID_VALUE => OpenGLError::InvalidValue,
            gl::INVALID_OPERATION => OpenGLError::InvalidOperation,
            gl::INVALID_FRAMEBUFFER_OPERATION => OpenGLError::InvalidFramebufferOperation,
            gl::OUT_OF_MEMORY => OpenGLError::OutOfMemory,
            gl::STACK_UNDERFLOW => OpenGLError::StackUnderflow,
            gl::STACK_OVERFLOW => OpenGLError::StackOverflow,
            gl::NO_ERROR => panic!("could not convert GL_NO_ERROR into an OpenGL error"),
            other => panic!("could not convert {other} an OpenGL error"),
        }
    }
}


// ---------------------------------------------------------------------------------------------------------------------


/// Represents the ways that raw data can be passed to OpenGL.
#[derive(Debug, Clone)]
pub enum RawData<'a> {
    /// This raw data is a reference to existing bytes.
    Ref(&'a [u8]),
    /// This raw data was freshly allocated.
    Vec(Vec<u8>),
}

// Wrapping a slice of bytes with a RawData
impl<'a> From<&'a [u8]> for RawData<'a> {
    fn from(value: &'a [u8]) -> Self {
        RawData::Ref(value)
    }
}

// Wrapping a vec of bytes with a RawData
impl<'a> From<Vec<u8>> for RawData<'a> {
    fn from(value: Vec<u8>) -> Self {
        RawData::Vec(value)
    }
}

// As long as we have either a reference to bytes or our own buffer of bytes, we can return a reference to bytes.
impl<'a> AsRef<[u8]> for RawData<'a> {
    fn as_ref(&self) -> &[u8] {
        match self {
            RawData::Ref(arr) => arr,
            RawData::Vec(vec) => vec,
        }
    }
}

// As long as a type is "plain old data" (https://docs.rs/bytemuck/1.13.1/bytemuck/trait.Pod.html), then we can safely
// reinterpret a reference to it as a byte slice. And, since `Pod` has its own blanket implementation for arrays, this
// means that arrays of `Pods` now also implement `Into<RawData>`!
impl<'a, T> From<&'a T> for RawData<'a>
where
    T: Pod + 'a,
{
    fn from(value: &'a T) -> Self {
        Self::Ref(bytes_of(value))
    }
}

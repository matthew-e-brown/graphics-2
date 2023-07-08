use bytemuck::{bytes_of, Pod};
pub use gl as bindings;
use gl::types::*;
use thiserror::Error;

pub mod buffers;
pub mod shaders;
pub mod vao;


/// The error returned when attempting to convert a [`GLenum`] into an actual enum fails due to the given `GLenum` value
/// not being a valid variant.
#[derive(Error, Debug)]
#[error("failed to match `GLenum` value '{0}' to an enum variant")]
pub struct GLEnumConversionError(GLenum);


// ---------------------------------------------------------------------------------------------------------------------


/// Declares a Rust macro with fields that map to specific [`GLenum`] values.
///
/// Enums are automatically declared with `From<$name> for GLenum` and `TryFrom<GLenum> for $name` implementations. By
/// default, [`Display`][std::fmt::Display] is also implemented, simply mapping each variant to a string of their OpenGL
/// name (e.g., `"GL_ARRAY_BUFFER"`). To implement `Display` yourself, include the `#[no_display]` attribute as the
/// **first attribute** on the enum (before doc comments, too).
///
/// # Syntax
///
/// Simply declare a regular enum, but with additional "match arms" for the OpenGL names. Exclude the `GL_` prefix on
/// the OpenGL names.
///
/// ```
/// gl_enum! {
///     /// Acceptable values for [buffer][Buffer] binding targets.
///     #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
///     pub enum BufferType {
///         /// Buffer target for vertex attributes.
///         ArrayBuffer => ARRAY_BUFFER,
///
///         /* -- snip -- */
///
///         /// Buffer target for uniform block storage.
///         UniformBuffer => UNIFORM_BUFFER,
///     }
/// }
/// ```
macro_rules! gl_enum {
    // Empty case to stop errors before typing any of the variants
    (
        $(#[$enum_attrs:meta])*
        $vis:vis enum $enum_name:ident {}
    ) => {
        $(#[$enum_attrs])*
        $vis enum $enum_name {}
    };

    // Base implementation, excludes display trait
    (
        #[no_display]
        $(#[$enum_attrs:meta])*
        $vis:vis enum $enum_name:ident {
            $(
                $(#[$field_attrs:meta])*
                $field_name:ident => $gl_name:ident
            ),*$(,)?
        }
    ) => {
        $(#[$enum_attrs])*
        $vis enum $enum_name {
            $(
                $(#[$field_attrs])*
                $field_name
            ),*
        }

        impl From<$enum_name> for GLenum {
            fn from(value: $enum_name) -> Self {
                match value {
                    $( $enum_name::$field_name => gl::$gl_name ),*
                }
            }
        }

        impl TryFrom<GLenum> for $enum_name {
            type Error = crate::GLEnumConversionError;

            fn try_from(value: GLenum) -> Result<Self, Self::Error> {
                match value {
                    $( gl::$gl_name => Ok(Self::$field_name), )*
                    other => Err(crate::GLEnumConversionError(other)),
                }
            }
        }
    };

    // Main implementation, calls base without display then implements display
    (
        $(#[$enum_attrs:meta])*
        $vis:vis enum $enum_name:ident {
            $(
                $(#[$field_attrs:meta])*
                $field_name:ident => $gl_name:ident
            ),*$(,)?
        }
    ) => {
        gl_enum! {
            #[no_display]
            $(#[$enum_attrs])*
            $vis enum $enum_name {
                $(
                    $(#[$field_attrs])*
                    $field_name => $gl_name
                ),*
            }
        }

        impl std::fmt::Display for $enum_name {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.write_str(match self {
                    $( $enum_name::$field_name => concat!("GL_", stringify!($gl_name)) ),*
                })
            }
        }
    };
}

pub(crate) use gl_enum;


// ---------------------------------------------------------------------------------------------------------------------


gl_enum! {
    #[no_display]
    /// An error from OpenGL.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub enum OpenGLError {
        /// An unacceptable value is specified for an enumerated argument.
        ///
        /// The offending command is ignored and has no other side effect than to set the error flag.
        InvalidEnum => INVALID_ENUM,
        /// A numeric argument is out of range.
        ///
        /// The offending command is ignored and has no other side effect than to set the error flag.
        InvalidValue => INVALID_VALUE,
        /// The specified operation is not allowed in the current state.
        ///
        /// The offending command is ignored and has no other side effect than to set the error flag.
        InvalidOperation => INVALID_OPERATION,
        /// The framebuffer object is not complete.
        ///
        /// The offending command is ignored and has no other side effect than to set the error flag.
        InvalidFramebufferOperation => INVALID_FRAMEBUFFER_OPERATION,
        /// There is not enough memory left to execute the command.
        ///
        /// The state of the GL is undefined, except for the state of the error flags, after this error is recorded.
        OutOfMemory => OUT_OF_MEMORY,
        /// An attempt has been made to perform an operation that would cause an internal stack to underflow.
        StackUnderflow => STACK_UNDERFLOW,
        /// An attempt has been made to perform an operation that would cause an internal stack to overflow.
        StackOverflow => STACK_OVERFLOW,
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
            Self::InvalidEnum => "an unacceptable value is specified for an enumerated argument",
            Self::InvalidValue => "a numeric argument is out of range",
            Self::InvalidOperation => "the specified operation is not allowed in the current state",
            Self::InvalidFramebufferOperation => "the framebuffer object is not complete",
            Self::OutOfMemory => "there is not enough memory left to execute the command",
            Self::StackUnderflow => {
                "an attempt has been made to perform an operation that would cause an internal stack to underflow"
            },
            Self::StackOverflow => {
                "an attempt has been made to perform an operation that would cause an internal stack to overflow"
            },
        })
    }
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
            // unwrap: we know anything else returned by `GetError` will be valid
            Some(err.try_into().unwrap())
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

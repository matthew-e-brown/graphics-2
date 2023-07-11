//! Ah, crap. There is a MASSIVE PROBLEM with this [`BindGuard`] approach. Originally, I modelled it after how
//! [`std::sync::Mutex`] and [`std::cell::Ref`]/[`std::cell::Ref`] worked: you would grab a temporary variable to allow
//! one to actually access the inner object, and you would require a `&mut` reference to be able to mutate it.
//! However... the fatal flaw is that there is no practical way to accommodate for the fact that binding a new buffer to
//! the same target will unbind the first one. That means that having a `BoundBuffer` (or another other
//! `Bindable::BoundType`) invariantly means that that buffer is bound; it may have been unbound by another one being
//! bound. As far as I can think of, there is no possible way, using just Rust's aliasing strictness and type system, to
//! ensure that a `BoundBuffer` remains valid.
//!
//! The only things I can think of to fix this issue are:
//!
//! - Maintain some global state in Rust that keeps track of the currently bound buffers, VAOs, framebuffers, and more.
//!   That would allow a `Buffer` object to quickly check that it is safe to bind itself, and return an `Err` if
//!   something is already bound that does not want to be overwritten. **Downside**: maintaining an entire second "copy"
//!   of OpenGL's internal state sounds like a recipe for trouble. Perhaps it wouldn't be *too* crazy, but there are
//!   *tons* of different types of objects that can be bound and different ways to use them.
//! - Query OpenGL before every bind to make sure that something isn't already bound. **Downside**: basically doubles
//!   the amount of OpenGL calls we have to make to the GPU, which doesn't sound very performant at all. It also
//!   completely eliminates the ability to have the user *intentionally* leave a buffer bound while dropping the
//!   `BindGuard`, when they want to rebind a second buffer without requiring an unnecessary `bindBuffer(0)` call in
//!   between.
//!
//! Perhaps there could be some other way, like making the binding targets themselves a consumable resource that gets
//! borrowed and given back. But at that point, we're back to trying to have some global state that would need to be
//! passed around everywhere.
//!
//! I think that, perhaps, the best thing to do would be to give up on this level of abstraction over OpenGL for this
//! library. I will commit these changes, so that the code is there, but I will half-restart this library as a closer
//! 1-to-1 of the actual OpenGL functions. I will write RAII wrappers for things that get `gen*`'ed or `create*`'ed, but
//! not worry about whether or not a buffer or VAO is "mutable" or not.

use std::marker::PhantomData;
use std::mem::ManuallyDrop;
use std::ops::{Deref, DerefMut};

use bytemuck::{bytes_of, Pod};
pub use gl as bindings;
use gl::types::*;
use thiserror::Error;

mod buffers;
mod shaders;
mod vao;

pub use buffers::*;
pub use shaders::*;
pub use vao::*;


/// The error returned when attempting to convert a [`GLenum`] into an actual enum fails due to the given `GLenum` value
/// not being a valid variant.
#[derive(Error, Debug)]
#[error("failed to match `GLenum` value '{0}' to an enum variant")]
pub struct EnumConversionError(GLenum);


/// An error returned when OpenGL fails to create an object.
///
/// This error is returned by several functions, and contains no extra details. It corresponds to when an
/// object-creation function returns zero (e.g., [`glCreateShader`] and [`glCreateProgram`]).
///
/// [`glCreateShader`]: https://registry.khronos.org/OpenGL-Refpages/gl4/html/glCreateShader.xhtml
/// [`glCreateProgram`]: https://registry.khronos.org/OpenGL-Refpages/gl4/html/glCreateProgram.xhtml
#[derive(Error, Debug)]
#[error("OpenGL could not create {0} object")]
pub struct ObjectCreationError(&'static str);
// note: {0} should include article ("a shader" object) ("an ..." object)


// ---------------------------------------------------------------------------------------------------------------------


pub trait Bindable {
    type BoundType;
}


/// An RAII implementation of a scoped bind for an OpenGL object.
///
/// Many objects in OpenGL need to be bound before they can be used. For example, calling [`glBufferData`] initializes
/// the data store for the currently bound buffer, set with [`glBindBuffer`]; calling `glBufferData` before calling
/// `glBindBuffer` is erroneous. In order to encapsulate this requirement as part of the type system, any struct that
/// needs to be bound before it can be used will have a `bind` or `use` method that returns a second, "bound" version of
/// itself; that second version will have the methods that require a binding to use. For example, [`Buffer::bind`]
///
/// [`glBufferData`]: https://registry.khronos.org/OpenGL-Refpages/gl4/html/glBufferData.xhtml
/// [`glBindBuffer`]: https://registry.khronos.org/OpenGL-Refpages/gl4/html/glBindBuffer.xhtml
pub struct BindGuard<'a, T: Bindable> {
    inner: T::BoundType,
    unbind: ManuallyDrop<Box<dyn FnOnce() + 'a>>,
    marker: PhantomData<&'a T>,
}

impl<'a, T: Bindable> BindGuard<'a, T> {
    pub fn new<F: FnOnce() + 'a>(inner: T::BoundType, unbind: F) -> Self {
        Self {
            inner,
            unbind: ManuallyDrop::new(Box::new(unbind)),
            marker: PhantomData,
        }
    }
}

impl<'a, T: Bindable> Drop for BindGuard<'a, T> {
    fn drop(&mut self) {
        // SAFETY: "It is your responsibility to ensure that this `ManuallyDropped` is not used again", say the Rust
        // docs. This is the **only** place it is ever used, as guaranteed by the private field. Additionally, this is
        // called immediately before it is dropped.
        let unbind = unsafe { ManuallyDrop::take(&mut self.unbind) };
        (unbind)();
    }
}

impl<'a, T: Bindable> Deref for BindGuard<'a, T> {
    type Target = T::BoundType;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

// ----------------------------------------------

/// The mutable version of a [`BindGuard`].
pub struct BindGuardMut<'a, T: Bindable> {
    inner: T::BoundType,
    unbind: ManuallyDrop<Box<dyn FnOnce() + 'a>>,
    marker: PhantomData<&'a mut T>,
}

impl<'a, T: Bindable> BindGuardMut<'a, T> {
    pub fn new<F: FnOnce() + 'a>(inner: T::BoundType, unbind: F) -> Self {
        Self {
            inner,
            unbind: ManuallyDrop::new(Box::new(unbind)),
            marker: PhantomData,
        }
    }
}

impl<'a, T: Bindable> Drop for BindGuardMut<'a, T> {
    fn drop(&mut self) {
        // SAFETY: see `BindGuard::Drop` impl.
        let unbind = unsafe { ManuallyDrop::take(&mut self.unbind) };
        (unbind)();
    }
}

impl<'a, T: Bindable> Deref for BindGuardMut<'a, T> {
    type Target = T::BoundType;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<'a, T: Bindable> DerefMut for BindGuardMut<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}


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
            type Error = crate::EnumConversionError;

            fn try_from(value: GLenum) -> Result<Self, Self::Error> {
                match value {
                    $( gl::$gl_name => Ok(Self::$field_name), )*
                    other => Err(crate::EnumConversionError(other)),
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

// Make available to the rest of the crate
use gl_enum;


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

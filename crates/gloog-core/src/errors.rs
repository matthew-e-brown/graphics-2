use gl::types::*;
use thiserror::Error;


/// The error returned when attempting to convert a raw [`GLenum`] into an actual Rust enum fails. This can happen when
/// the given `GLenum` value does not match any variants.
#[derive(Error, Debug)]
#[error("could not convert `GLenum` value '{original_value:#X}' into enum of type `{attempted_type}`")]
pub struct EnumConversionError {
    original_value: GLenum,
    attempted_type: &'static str,
}

impl EnumConversionError {
    pub(crate) const fn new(value: GLenum, name: &'static str) -> Self {
        Self {
            original_value: value,
            attempted_type: name,
        }
    }
}


/// The error returned when attempting to convert a raw [`GLbitfield`] into an actual Rust struct fails. This can happen
/// when the given `GLbitfield` value does not
#[derive(Error, Debug)]
#[error("could not convert `GLbitfield` value '{original_value:#b}' into struct of type `{attempted_type}`")]
pub struct BitFieldConversionError {
    original_value: GLbitfield,
    attempted_type: &'static str,
}

impl BitFieldConversionError {
    pub(crate) const fn new(value: GLbitfield, name: &'static str) -> Self {
        Self {
            original_value: value,
            attempted_type: name,
        }
    }
}


/// An error returned when OpenGL itself fails to create an object.
///
/// This error is returned by several functions, and contains no extra details. It corresponds to when an
/// object-creation function (e.g., [`glCreateShader`] and [`glCreateProgram`]) returns zero.
///
/// [`glCreateShader`]: https://registry.khronos.org/OpenGL-Refpages/gl4/html/glCreateShader.xhtml
/// [`glCreateProgram`]: https://registry.khronos.org/OpenGL-Refpages/gl4/html/glCreateProgram.xhtml
#[derive(Error, Debug)]
#[error("OpenGL could not create {0} object")]
// note: {0} should *include* article ("a shader" object) ("an ..." object)
pub struct ObjectCreationError(&'static str);

impl ObjectCreationError {
    pub(crate) const fn new(type_name: &'static str) -> Self {
        Self(type_name)
    }
}

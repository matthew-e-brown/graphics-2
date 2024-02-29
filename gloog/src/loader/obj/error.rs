use std::io;

use gloog_core::bindings::types::GLuint;
use thiserror::Error;

use crate::loader::{fmt_line_range, LineRange};

// cspell:words newmtl

pub type ObjResult<T> = Result<T, ObjLoadError>;
pub type MtlResult<T> = Result<T, MtlLoadError>;


#[rustfmt::skip]
#[derive(Error, Debug)]
pub enum ObjLoadError {
    #[error("failed to read from file:\n{0}")]
    IOReadError(#[from] io::Error),

    #[error("failed to open mtl file:\n{0}")]
    MtlOpenError(io::Error),

    #[error("error parsing mtl file:\n{0}")]
    MtlError(MtlLoadError),

    #[error("unknown material {name} on {}", fmt_line_range(.lines))]
    UnknownMaterial { lines: LineRange, name: String },

    #[error("'{directive}' directive on {} has invalid float(s)", fmt_line_range(.lines))]
    VertexParseError { lines: LineRange, directive: &'static str },

    #[error("'{directive}' directive on {} has {n} of required {min} floats", fmt_line_range(.lines))]
    VertexTooSmall { lines: LineRange, directive: &'static str, n: usize, min: usize },

    #[error("'{directive}' directive on {} has {n} floats, but max is {max}", fmt_line_range(.lines))]
    VertexTooLarge { lines: LineRange, directive: &'static str, n: usize, max: usize },

    #[error("too many unique vertex attributes, maximum number is {}", GLuint::MAX - 1)]
    VertexDataOverflow,

    #[error("'f' directive on {} has invalid vertex index", fmt_line_range(.lines))]
    FaceIndexParseError { lines: LineRange },

    #[error("'f' directive on {} has {n} vertices, but minimum is 3", fmt_line_range(.lines))]
    FaceTooFewIndices { lines: LineRange, n: usize },

    #[error("'f' directive on {} has {n} vertices, but maximum is {}", fmt_line_range(.lines), u16::MAX)]
    FaceTooManyIndices { lines: LineRange, n: usize },

    #[error("'f' directive on {} has inconsistent v/vt/vn configuration", fmt_line_range(.lines))]
    FaceMismatchedIndexConfig { lines: LineRange },

    #[error("'f' directive on {} references vertex data '{list}' at index, out of range for list of length {len}", fmt_line_range(.lines))]
    FaceIndexOutOfRange { lines: LineRange, list: &'static str, idx: isize, len: usize },

    #[error("unknown directive '{directive}' on {}", fmt_line_range(.lines))]
    UnknownDirective { lines: LineRange, directive: String },
}

#[derive(Error, Debug)]
pub enum MtlLoadError {
    #[error("failed to read from file:\n{0}")]
    IOReadError(io::Error),

    #[error("failed to open file:\n{0}")]
    IOOpenError(io::Error),

    #[error("encountered material property before 'newmtl' directive on {}", fmt_line_range(.0))]
    BeforeName(LineRange),

    #[error("'newmtl' directive is missing a name on {}", fmt_line_range(.0))]
    MissingName(LineRange),

    #[error("encountered unsupported '{1}' color on {}", fmt_line_range(.0))]
    UnsupportedColorFormat(LineRange, &'static str),

    #[error("invalid color on {}", fmt_line_range(.0))]
    InvalidColor(LineRange),

    #[error("invalid scalar on {}", fmt_line_range(.0))]
    InvalidScalar(LineRange),

    #[error("duplicate property {1} on {}", fmt_line_range(.0))]
    DuplicateProperty(LineRange, &'static str),
}


impl<T> From<ObjLoadError> for Result<T, ObjLoadError> {
    fn from(value: ObjLoadError) -> Self {
        Err(value)
    }
}

impl<T> From<MtlLoadError> for Result<T, MtlLoadError> {
    fn from(value: MtlLoadError) -> Self {
        Err(value)
    }
}

impl From<MtlLoadError> for ObjLoadError {
    fn from(value: MtlLoadError) -> Self {
        match value {
            MtlLoadError::IOReadError(inner) => Self::IOReadError(inner),
            other => Self::MtlError(other),
        }
    }
}


/// [`Range`][std::ops::Range] is not [`Copy`], and [`Clone`] is not `const`.
const fn clone_range(r: &LineRange) -> LineRange {
    r.start..r.end
}

// Helper functions for quick construction of error values
impl ObjLoadError {
    #[inline(always)]
    pub(super) fn unknown_mtl(lines: &LineRange, name: &str) -> Self {
        let lines = clone_range(lines);
        Self::UnknownMaterial { lines, name: name.to_owned() }
    }

    #[inline(always)]
    pub(super) fn v_parse_err(lines: &LineRange, directive: &'static str) -> Self {
        let lines = clone_range(lines);
        Self::VertexParseError { lines, directive }
    }

    #[inline(always)]
    pub(super) fn v_too_small(lines: &LineRange, directive: &'static str, n: usize, min: usize) -> Self {
        let lines = clone_range(lines);
        Self::VertexTooSmall { lines, directive, n, min }
    }

    #[inline(always)]
    pub(super) fn v_too_large(lines: &LineRange, directive: &'static str, n: usize, max: usize) -> Self {
        let lines = clone_range(lines);
        Self::VertexTooLarge { lines, directive, n, max }
    }

    #[inline(always)]
    pub(super) fn f_parse_err(lines: &LineRange) -> Self {
        let lines = clone_range(lines);
        Self::FaceIndexParseError { lines }
    }

    #[inline(always)]
    pub(super) fn f_too_few(lines: &LineRange, n: usize) -> Self {
        let lines = clone_range(lines);
        Self::FaceTooFewIndices { lines, n }
    }

    pub(super) fn f_too_many(lines: &LineRange, n: usize) -> Self {
        let lines = clone_range(lines);
        Self::FaceTooFewIndices { lines, n }
    }

    #[inline(always)]
    pub(super) fn f_mismatched(lines: &LineRange) -> Self {
        let lines = clone_range(lines);
        Self::FaceMismatchedIndexConfig { lines }
    }

    #[inline(always)]
    pub(super) fn f_index_range(lines: &LineRange, list: &'static str, idx: isize, len: usize) -> Self {
        let lines = clone_range(lines);
        Self::FaceIndexOutOfRange { lines, list, idx, len }
    }

    #[inline(always)]
    pub(super) fn unknown(lines: &LineRange, directive: &str) -> Self {
        let lines = clone_range(lines);
        Self::UnknownDirective {
            lines,
            directive: directive.to_owned(),
        }
    }
}

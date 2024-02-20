//! Module for loading models from OBJ files, optionally paired with MTL material files.
//!
//! Only all-polygonal models are supported are the moment (no free-form surfaces). These are those that use only `f`,
//! no `curv` or `surf` statements. Unsupported statements are simply ignored, though a warning is produced.

mod mtl;

use std::collections::BTreeMap;
use std::fs::File;
use std::io::{self, BufReader};
use std::path::Path;
use std::sync::Arc;

use arrayvec::ArrayVec;
use gloog_math::{Vec2, Vec3};
use log::{debug, log, trace};
use thiserror::Error;

use self::mtl::MtlMaterial;
use super::{fmt_line_range, lines_escaped, LineRange};

// cspell:words curv interp stech ctech scrv cstype bmat usemtl mtllib maplib usemap

// Source for OBJ and MTL specs:
// - https://www.uhu.es/francisco.moreno/gii_rv/practicas/practica08/OBJ_SPEC.PDF
// - also: https://paulbourke.net/dataformats/obj/ (missing math section)
// - https://paulbourke.net/dataformats/mtl/

// ---------------------------------------------------------------------------------------------------------------------

#[rustfmt::skip]
#[derive(Error, Debug)]
pub enum ObjParseError {
    #[error("failed to read from file:\n{0:?}")]
    IOError(#[from] io::Error),

    #[error("'{directive}' directive on {} has invalid float(s)", fmt_line_range(.lines))]
    InvalidFloats { lines: LineRange, directive: &'static str },

    #[error("'{directive}' directive on {} has {n} of required {min} floats", fmt_line_range(.lines))]
    NotEnoughFloats { lines: LineRange, directive: &'static str, n: usize, min: usize },

    #[error("'{directive}' directive on {} has {n} floats, but max is {max}", fmt_line_range(.lines))]
    TooManyFloats { lines: LineRange, directive: &'static str, n: usize, max: usize },

    #[error("unknown directive '{directive}' on {}", fmt_line_range(.lines))]
    UnknownDirective { lines: LineRange, directive: String },
}

// Helper functions for quick construction of error values
#[rustfmt::skip]
impl ObjParseError {
    #[inline(always)]
    fn invalid<T>(lines: &LineRange, directive: &'static str) -> Result<T, Self> {
        Err(Self::InvalidFloats { lines: lines.clone(), directive })
    }

    #[inline(always)]
    fn not_enough<T>(lines: &LineRange, directive: &'static str, n: usize, min: usize) -> Result<T, Self> {
        Err(Self::NotEnoughFloats { lines: lines.clone(), directive, n, min })
    }

    #[inline(always)]
    fn too_many<T>(lines: &LineRange, directive: &'static str, n: usize, max: usize) -> Result<T, Self> {
        Err(Self::TooManyFloats { lines: lines.clone(), directive, n, max })
    }

    #[inline(always)]
    fn unknown<T>(lines: &LineRange, directive: &str) -> Result<T, Self> {
        Err(Self::UnknownDirective { lines: lines.clone(), directive: directive.to_owned() })
    }
}

// ---------------------------------------------------------------------------------------------------------------------


/// Raw data from an OBJ file, ready to be converted into an actual OpenGL-ready model.
#[derive(Debug, Default)]
pub struct ObjModel {
    /// Geometric vertex data. Created by `v` statements.
    vertices: Vec<Vec3>,
    /// Vertex normals. Created by `vn` statements.
    normals: Vec<Vec3>,
    /// Texture vertices/coordinates. Created by `vt` statements.
    ///
    /// The spec says that `vn` statements may specify `u`, `v`, *and* `w` components. It seems that this means that it
    /// supports 3D textures (eg., for volumetric effects), but it also says that "`w` is a value for the depth of the
    /// texture," which implies that it's more for mipmapping. We'll handle that part ourselves, so we omit the `w`. In
    /// cases where `v` is omitted, it defaults to zero.
    tex_coords: Vec<Vec2>,
    /// List of all groups encountered in the file.
    faces: Vec<FaceElement>,
    /// List of all materials in the file, indexed by name.
    materials: BTreeMap<Arc<str>, MtlMaterial>,
}


/// A polygonal face (polygon) in an OBJ-file model.
#[derive(Debug)]
pub struct FaceElement {
    /// The name of the material that this face uses.
    mtl_name: Option<Arc<str>>,
    /// A list of 0-based index into geometric vertex data.
    vertices: Vec<usize>,
    /// A list of 0-based indices into vertex texture coordinate data, if applicable.
    tex_coords: Option<Vec<usize>>,
    /// A list of 0-based indices into normal data, if applicable.
    normals: Option<Vec<usize>>,
}


/// Current parser state as we run through the file.
///
/// OBJ parsing is state-based. Lines are "directives" which either introduce new vertices or modify current states.
/// States include things like which group things are a part of, current material information, and so on.
#[derive(Debug, Default)]
struct ParseState {
    cur_material: Option<Arc<str>>,
}


pub fn load(path: impl AsRef<Path>) -> Result<ObjModel, ObjParseError> {
    let mut data = ObjModel::default(); // Final data that gets returned
    let mut state = ParseState::default(); // Other state that changes as we parse

    let path = path.as_ref();
    let file = BufReader::new(File::open(path)?);
    let mut lines = lines_escaped(file);

    while let Some(read_result) = lines.next() {
        let (line_nums, line) = read_result?;

        // Trim lines, ignoring comments
        let line = {
            let before_hash = line.find('#').map(|i| 0..i).unwrap_or(0..line.len());
            let uncommented = &line[before_hash];
            uncommented.trim()
        };

        // If our line is empty after removing the comment, we can ignore it
        if line.len() == 0 {
            continue;
        }

        trace!("parsing line contents: `{line}`");

        // Get the first thing before the first space (can unwrap because we just checked that the line was not
        // zero-length, and we did so after trimming, which guarantees that there is at least one thing pre-whitespace).
        let directive = line.split_whitespace().nth(0).unwrap();
        let rest = line[directive.len()..].trim(); // rest of the line

        match directive {
            "v" => read_v(rest, &mut data.vertices, &line_nums)?,
            "vn" => read_vn(rest, &mut data.normals, &line_nums)?,
            "vt" => read_vt(rest, &mut data.tex_coords, &line_nums)?,
            "f" => unimplemented!("'f' directive"),
            "mtllib" => unimplemented!("'mtllib' directive"),
            "usemtl" => unimplemented!("'usemtl' directive"),
            _ => check_other(directive, &line_nums)?,
        }
    }

    debug!(
        "finished parsing file {}. found {} vertices, {} normals, and {} tex-coords used by {} faces.",
        path.display(),
        data.vertices.len(),
        data.normals.len(),
        data.tex_coords.len(),
        data.faces.len(),
    );

    todo!();
}


/// Parses the body of a `v` directive from an OBJ file and inserts its resultant vector into the given list. `lines` is
/// needed for error-reporting.
fn read_v(text: &str, data: &mut Vec<Vec3>, lines: &LineRange) -> Result<(), ObjParseError> {
    // Take at most four; some files seem to specify extra `1` values after the w to force some viewers to get the
    // message (that's my guess anyways).
    let floats = text.split_whitespace().take(4).map(|s| s.parse());
    let floats: ArrayVec<f32, 4> = match floats.collect() {
        Ok(vec) => vec,
        Err(_) => return ObjParseError::invalid(lines, "v"),
    };

    // xyz are required; w is optional and defaults to 1. In OBJ, w is only used for free-form curves and surfaces:
    // there are no homogeneous coordinates/transformations within OBJ files; everything is simply stored in 3D space.
    // Meaning, we should never run into a case where `w` it isn't 1. If we do, simply emit a small warning/notice that
    // the file _may_ contain unsupported features and that we're ignoring it.
    if floats.len() < 3 {
        ObjParseError::not_enough(lines, "v", floats.len(), 3)
    } else {
        if floats.len() == 4 && floats[3] != 1.0 {
            debug!(
                "ignoring non-1.0 'w' component of '{}', since 'w' only affects free-form geometry (unsupported)",
                text
            );
        }

        // Take the first three and convert them into an array, then into a vector; safe to unwrap because we already
        // know that we correctly parsed 3-4 floats.
        let arr: [f32; 3] = floats[0..3].try_into().unwrap();
        let vec: Vec3 = arr.into();

        trace!("parsed vertex {vec:?}");

        data.push(vec);
        Ok(())
    }
}

/// Parses the body of a `vn` directive from an OBJ file.
fn read_vn(text: &str, data: &mut Vec<Vec3>, lines: &LineRange) -> Result<(), ObjParseError> {
    // Take as many as they provided, and expect exactly three. The spec says that there should always be three.
    let floats = text.split_whitespace().map(|s| s.parse());
    let floats: Vec<f32> = match floats.collect() {
        Ok(vec) => vec,
        Err(_) => return ObjParseError::invalid(lines, "vn"),
    };

    if floats.len() < 3 {
        ObjParseError::not_enough(lines, "vn", floats.len(), 3)
    } else if floats.len() > 3 {
        ObjParseError::too_many(lines, "vn", floats.len(), 3)
    } else {
        // We know there's exactly three now
        let arr: [f32; 3] = floats.try_into().unwrap();
        let vec: Vec3 = arr.into();

        trace!("parsed normal {vec:?}");

        data.push(vec);
        Ok(())
    }
}

/// Parses the body of a `vn` directive from an OBJ file.
fn read_vt(text: &str, data: &mut Vec<Vec2>, lines: &LineRange) -> Result<(), ObjParseError> {
    let floats = text.split_whitespace().map(|s| s.parse());
    let floats: Vec<f32> = match floats.collect() {
        Ok(vec) => vec,
        Err(_) => return ObjParseError::invalid(lines, "vt"),
    };

    let vec: Vec2 = match floats.len() {
        0 => return ObjParseError::not_enough(lines, "vt", floats.len(), 1),
        1 => [floats[0], 0.0].into(),
        2 => [floats[0], floats[1]].into(),
        3 => {
            debug!("found texture coord with 3rd dimension; 3rd dimension will be ignored");
            [floats[0], floats[1]].into()
        },
        n => return ObjParseError::too_many(lines, "vt", n, 3),
    };

    trace!("parsed tex-coord {vec:?}");

    data.push(vec);
    Ok(())
}


/// Checks whether or not an unsupported directive can be skipped gracefully and logs it if applicable.
fn check_other(directive: &str, lines: &LineRange) -> Result<(), ObjParseError> {
    // For unsupported features, they are split into ones that we can gracefully ignore vs. ones that the
    // user may expect to make a difference.
    let level = match directive {
        // Grouping isn't needed because we don't do any fancy visualization; just rendering.
        "g" | "o" => log::Level::Trace,
        // We only support `mtl` files through `mtllib`, not others (mostly because there is basically zero
        // documentation on how these two work).
        "maplib" | "usemap" => log::Level::Error,
        // Smoothing we want the user to know isn't working. Merge groups are part of free-form surfaces.
        "s" => log::Level::Warn,
        "mg" => log::Level::Error,
        // Point, line, and curve elements are usually non-visible anyways (infinitely thin), but we don't
        // want the user to expect us to handle those
        "p" | "l" | "curv" | "curv2" => log::Level::Warn,
        // Parameter space vertices and free form surfaces are not supported at all.
        "vp" | "surf" => log::Level::Error,
        // Their associated controls are therefore not supported either, but they're less severe.
        "cstype" | "deg" | "step" | "bmat" => log::Level::Warn,
        "parm" | "trim" | "hole" | "scrv" | "sp" | "con" => log::Level::Warn,
        "end" => log::Level::Trace,
        // Display/render attributes other than usemtl and mtllib are also unsupported.
        "bevel" => log::Level::Warn,
        // These could maybe be implemented in the future? For toggling vertex-colour interpolation?
        "c_interp" | "d_interp" => log::Level::Warn,
        // Level-of-detail, shadow and ray tracing, etc.
        "lod" | "shadow_obj" | "trace_obj" | "ctech" | "stech" => log::Level::Error,
        // Anything else is a flat-out unknown directive.
        _ => return ObjParseError::unknown(lines, directive),
    };

    log!(level, "ignoring *.obj directive '{directive}'; feature unsupported");
    Ok(())
}

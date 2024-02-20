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
use log::{debug, trace, warn};
use thiserror::Error;

use self::mtl::MtlMaterial;
use super::{fmt_line_range, lines_escaped, LineRange};

// cspell:words curv interp stech ctech usemtl mtllib

// Source for OBJ and MTL specs:
// - https://www.uhu.es/francisco.moreno/gii_rv/practicas/practica08/OBJ_SPEC.PDF
// - also: https://paulbourke.net/dataformats/obj/ (missing math section)
// - https://paulbourke.net/dataformats/mtl/

// ---------------------------------------------------------------------------------------------------------------------

#[rustfmt::skip]
#[derive(Error, Debug)]
pub enum ObjLoadError {
    #[error("failed to read from file:\n{0:?}")]
    IOError(#[from] io::Error),

    #[error("'{directive}' directive at {} has invalid float", fmt_line_range(.lines))]
    InvalidFloats { lines: LineRange, directive: &'static str },

    #[error("'{directive}' directive at {} has {n} of required {min} floats", fmt_line_range(.lines))]
    NotEnoughFloats { lines: LineRange, directive: &'static str, n: usize, min: usize },

    #[error("'{directive}' directive at {} has {n} floats, but max is {max}", fmt_line_range(.lines))]
    TooManyFloats { lines: LineRange, directive: &'static str, n: usize, max: usize },

    #[error("unknown directive '{directive}' at {}", fmt_line_range(.lines))]
    UnknownDirective { lines: LineRange, directive: String },
}


// Helper functions for quick construction of error values
#[rustfmt::skip]
impl ObjLoadError {
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
    groups: BTreeMap<Arc<str>, Vec<FaceElement>>,
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
    cur_group: Option<Arc<str>>,
    cur_material: Option<Arc<str>>,
}


pub fn load(path: impl AsRef<Path>) -> Result<ObjModel, ObjLoadError> {
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
            "f" => (),
            "g" => (),
            "o" => (),
            "usemtl" => (),
            "mtllib" => (),
            // Other
            "vp" | "p" | "l" | "s" | "mg" | "bevel" | "c_interp" | "d_interp" | "lod" | "shadow_obj" | "trace_obj"
            | "ctech" | "stech" => warn!("unsupported *.obj directive `{directive}`"),
            // ------
            _ => return ObjLoadError::unknown(&line_nums, directive),
        }
    }

    debug!(
        "finished parsing file {}. found {} vertices, {} normals, and {} tex-coords used by {} faces.",
        path.display(),
        data.vertices.len(),
        data.normals.len(),
        data.tex_coords.len(),
        data.groups.values().map(|g| g.len()).sum::<usize>(),
    );

    todo!();
}


/// Parses the body of a `v` directive from an OBJ file and inserts its resultant vector into the given list. `lines` is
/// needed for error-reporting.
fn read_v(text: &str, data: &mut Vec<Vec3>, lines: &LineRange) -> Result<(), ObjLoadError> {
    // Take at most four; some files seem to specify extra `1` values after the w to force some viewers to get the
    // message (that's my guess anyways).
    let floats = text.split_whitespace().take(4).map(|s| s.parse());
    let floats: ArrayVec<f32, 4> = match floats.collect() {
        Ok(vec) => vec,
        Err(_) => return ObjLoadError::invalid(lines, "v"),
    };

    // xyz are required; w is optional and defaults to 1. In OBJ, w is only used for free-form curves and surfaces:
    // there are no homogeneous coordinates/transformations within OBJ files; everything is simply stored in 3D space.
    // Meaning, we should never run into a case where `w` it isn't 1. If we do, simply emit a small warning/notice that
    // the file _may_ contain unsupported features and that we're ignoring it.
    if floats.len() < 3 {
        ObjLoadError::not_enough(lines, "v", floats.len(), 3)
    } else {
        if floats.len() == 4 && floats[3] != 1.0 {
            debug!("found vertex with non-1.0 'w' component {}; 'w' only affects free-form geometry, so it will be ignored", floats[3]);
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
fn read_vn(text: &str, data: &mut Vec<Vec3>, lines: &LineRange) -> Result<(), ObjLoadError> {
    // Take as many as they provided, and expect exactly three. The spec says that there should always be three.
    let floats = text.split_whitespace().map(|s| s.parse());
    let floats: Vec<f32> = match floats.collect() {
        Ok(vec) => vec,
        Err(_) => return ObjLoadError::invalid(lines, "vn"),
    };

    if floats.len() < 3 {
        ObjLoadError::not_enough(lines, "vn", floats.len(), 3)
    } else if floats.len() > 3 {
        ObjLoadError::too_many(lines, "vn", floats.len(), 3)
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
fn read_vt(text: &str, data: &mut Vec<Vec2>, lines: &LineRange) -> Result<(), ObjLoadError> {
    let floats = text.split_whitespace().map(|s| s.parse());
    let floats: Vec<f32> = match floats.collect() {
        Ok(vec) => vec,
        Err(_) => return ObjLoadError::invalid(lines, "vt"),
    };

    let vec: Vec2 = match floats.len() {
        0 => return ObjLoadError::not_enough(lines, "vt", floats.len(), 1),
        1 => [floats[0], 0.0].into(),
        2 => [floats[0], floats[1]].into(),
        3 => {
            debug!("found texture coord with 3rd dimension; it will be ignored");
            [floats[0], floats[1]].into()
        },
        n => return ObjLoadError::too_many(lines, "vt", n, 3),
    };

    trace!("parsed tex-coord {vec:?}");

    data.push(vec);
    Ok(())
}


// /// Reads a series of whitespace-separated items into an [`ArrayVec`] of the given capacity.
// fn read_to_array<T: FromStr, const N: usize>(text: &str) -> Result<ArrayVec<T, N>, T::Err> {
//     text.trim().split_whitespace().take(N).map(|str| str.parse()).collect()
// }

// fn index_to_unsigned(idx: isize, cur_len: usize) -> usize {
//     if idx > 0 {
//         (idx as usize) - 1
//     } else if idx < 0 {
//         cur_len - (idx.abs() as usize) - 1
//     } else {
//         panic!("encountered zero index");
//     }
// }

// /// Reads a series of whitespace-separated items into a [`Vec`] of the given capacity.
// fn read_indices(text: &str, cur_v_len: usize) -> Vec<usize> {
//     text.trim()
//         .split_whitespace()
//         .map(|str| index_to_unsigned(str.parse().unwrap(), cur_v_len))
//         .collect()
// }

// fn read_slashed_indices(text: &str, cur_v_len: usize, cur_vn_len: usize, cur_vt_len: usize) -> Vec<usize> {
//     // Parse `a/b/c`, with optional `b` and `c`, into a tuple
//     let parse = |text: &str| -> (usize, Option<usize>, Option<usize>) {
//         let mut nums = text.split('/').map(|n| n.parse::<isize>());
//         let v = index_to_unsigned(nums.next().unwrap().unwrap(), cur_v_len);
//         let vt = nums.next().map(|idx| index_to_unsigned(idx.unwrap(), cur_vn_len));
//         let vn = nums.next().map(|idx| index_to_unsigned(idx.unwrap(), cur_vt_len));
//         (v, vt, vn)
//     };

//     // Split on whitespace
//     let mut verts = text.trim().split_whitespace();

//     // Parse the first one first

//     todo!();
// }

// fn read_face_elem(text: &str, data: &ObjModel) -> FaceElement {
//     todo!()
// }

//! Module for loading models from OBJ files, optionally paired with MTL material files.

use std::fs::File;
use std::io::{self, BufReader};
use std::path::Path;
use std::str::FromStr;

use arrayvec::ArrayVec;
use log::{error, trace};
use thiserror::Error;

use super::{fmt_line_range, lines_escaped, LineRange};

// cspell:words curv interp stech ctech usemtl mtllib

// Source for OBJ and MTL specs:
// - https://www.uhu.es/francisco.moreno/gii_rv/practicas/practica08/OBJ_SPEC.PDF
// - also: https://paulbourke.net/dataformats/obj/ (missing math section)
// - https://paulbourke.net/dataformats/mtl/

// OBJ parsing is state-based. Lines are "directives" which either introduce new vertices or modify current states.
// States include things like which group things are a part of, current material information, and so on.


// =====================================================================================================================

// TODO: Make proper errors instead of unwrapping. Unwrapping is just for during tests.

// #[derive(Error, Debug)]
// pub enum ObjLoadError {
//     #[error("failed to read from file:\n{0:?}")]
//     IOError(#[from] io::Error),
//
//     #[error("invalid '{0}' directive at {}", fmt_line_range(.1))]
//     InvalidDirective(&'static str, LineRange),
//
//     #[error("unsupported directive: {0}")]
//     UnsupportedDirective(String),
// }

// =====================================================================================================================


/// Raw data from an OBJ file, ready to be converted into an actual OpenGL-ready model.
#[derive(Debug, Default)]
pub struct ObjData {
    /// Geometric vertex data. Created by `v` statements.
    v: Vec<ArrayVec<f32, 4>>,

    /// Texture vertices/coordinates. Created by `vt` statements. Similar to regular, geometric vertices, these may be
    /// 1D, 2D, or 3D; always stored as 3D.
    vt: Vec<ArrayVec<f32, 3>>,

    /// Parameter-space vertices / control points. Created by `vp` statements. May be 1D, 2D, or 3D; always stored as
    /// 3D.
    vp: Vec<ArrayVec<f32, 3>>,

    /// Vertex normals. Created by `vn` statements. Always 3D vectors.
    vn: Vec<ArrayVec<f32, 3>>,

    /// Points. Created by `p` statements.
    ///
    /// Note from spec: "although points cannot be shaded or rendered, they are used by other Advanced Visualizer
    /// programs."
    points: Vec<usize>,

    /// Lines (specifically polylines). Created by `l` statements.
    ///
    /// Note from spec: "although lines cannot be shaded or rendered, they are used by other Advanced Visualizer
    /// programs."
    lines: Vec<LineElement>,

    /// Face elements (polygons). Created by `f` statements.
    faces: Vec<FaceElement>,

    curves_1d: Vec<Curve>,

    curves_2d: Vec<Curve>,

    surface: Vec<Surface>,
}

/// A line (specifically a polyline) in an OBJ-file model.
#[derive(Debug)]
pub struct LineElement {
    v: Vec<usize>,
    vt: Option<Vec<usize>>,
}

/// A polygonal face (polygon) in an OBJ-file model.
#[derive(Debug)]
pub struct FaceElement {
    /// A list of 0-based index into geometric vertex data.
    v: Vec<usize>,
    /// A list of 0-based indices into vertex texture coordinate data, if applicable.
    vt: Option<Vec<usize>>,
    /// A list of 0-based indices into normal data, if applicable.
    vn: Option<Vec<usize>>,
}

// There are three steps involved in specifying a free-form curve or surface element.
// - Specify the type of curve or surface (basis matrix, Bezier, B-spline, Cardinal, or Taylor) using free-form
//   curve/surface attributes.
// - Describe the curve or surface with element statements.
// - Supply additional information, using free-form curve/surface body statements.

#[derive(Debug)]
pub enum Curve {
    Bezier {
        rational: bool,
        degree: u32,
    },
    BSpline {
        rational: bool,
        degree: u32,
    },
    Cardinal {
        rational: bool,
        degree: u32,
    },
    Taylor {
        rational: bool,
        degree: u32,
    },
    BasisMatrix {
        rational: bool,
        degree: u32,
        step: f32,
        mat: Vec<f32>,
    },
}

#[derive(Debug)]
pub enum Surface {
    Bezier {
        rational: bool,
        degree_u: u32,
        degree_v: u32,
    },
    BSpline {
        rational: bool,
        degree_u: u32,
        degree_v: u32,
    },
    Cardinal {
        rational: bool,
        degree_u: u32,
    },
    Taylor {
        rational: bool,
        degree_u: u32,
        degree_v: u32,
    },
    BasisMatrix {
        rational: bool,
        step_u: f32,
        step_v: f32,
        mat_u: Vec<f32>,
        mat_v: Vec<f32>,
    },
}


enum CSType {
    BasisMatrix,
    Bezier,
    BSpline,
    Cardinal,
    Taylor,
}

#[derive(Default)]
struct ObjState {
    cs_type: Option<CSType>,
    step_u: Option<f32>,
    step_v: Option<f32>,
    basis_mat_u: Vec<f32>,
    basis_mat_v: Vec<f32>,
    degree_u: Option<usize>,
    degree_v: Option<usize>,
}


pub fn load(path: impl AsRef<Path>) -> /* Result<ObjData, ObjLoadError> */ ObjData {
    let mut data = ObjData::default(); // Final data that gets returned
    let mut state = ObjState::default(); // Other state that changes as we parse
    let mut lines = lines_escaped(BufReader::new(File::open(path).unwrap()));

    while let Some(read_result) = lines.next() {
        let (_lines, line) = read_result.unwrap();

        trace!("parsing line: {line}");

        // /// Returns an [`Err`] containing [`ObjLoadError::InvalidDirective`] with the given directive string and current
        // /// line range.
        // macro_rules! invalid {
        //     ($directive:literal) => {
        //         Err(ObjLoadError::InvalidDirective($directive, lines))
        //     };
        // }

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

        // Get the first thing before the first space (can unwrap because we just checked that the line was not
        // zero-length, and we did so after trimming, which guarantees that there is at least one thing pre-whitespace).
        let directive = line.split_whitespace().nth(0).unwrap();
        let rest = line[directive.len()..].trim(); // rest of the line

        match directive {
            // vertex data: push directly into buffers
            "v" => data.v.push(read_to_array::<f32, 4>(rest).unwrap()),
            "vp" => data.vp.push(read_to_array::<f32, 3>(rest).unwrap()),
            "vn" => data.vn.push(read_to_array::<f32, 3>(rest).unwrap()),
            "vt" => data.vt.push(read_to_array::<f32, 3>(rest).unwrap()),
            // elements: read indices relative to current respective vertex buffer lengths
            "p" => data.points.extend(read_indices(rest, data.v.len())),
            "l" => data.lines.push(read_line_elem(rest, &data)),
            "f" => data.faces.push(read_face_elem(rest, &data)),
            "curv" => {},
            "curv2" => {},
            "surf" => {},
            // grouping
            "g" => {},
            "s" => {},
            "mg" => {},
            "o" => {},
            // display/render attributes
            "usemtl" => {},
            "mtllib" => {},
            "bevel" => {},
            "c_interp" => {},
            "d_interp" => {},
            "lod" => {},
            "shadow_obj" => {},
            "trace_obj" => {},
            "ctech" => {},
            "stech" => {},
            // ------
            other => panic!("unsupported directive {other}"),
        }
    }

    todo!();
}


/// Reads a series of whitespace-separated items into an [`ArrayVec`] of the given capacity.
fn read_to_array<T: FromStr, const N: usize>(text: &str) -> Result<ArrayVec<T, N>, T::Err> {
    text.trim().split_whitespace().take(N).map(|str| str.parse()).collect()
}

fn index_to_unsigned(idx: isize, cur_len: usize) -> usize {
    if idx > 0 {
        (idx as usize) - 1
    } else if idx < 0 {
        cur_len - (idx.abs() as usize) - 1
    } else {
        panic!("encountered zero index");
    }
}

/// Reads a series of whitespace-separated items into a [`Vec`] of the given capacity.
fn read_indices(text: &str, cur_v_len: usize) -> Vec<usize> {
    text.trim()
        .split_whitespace()
        .map(|str| index_to_unsigned(str.parse().unwrap(), cur_v_len))
        .collect()
}

fn read_slashed_indices(text: &str, cur_v_len: usize, cur_vn_len: usize, cur_vt_len: usize) -> Vec<usize> {
    // Parse `a/b/c`, with optional `b` and `c`, into a tuple
    let parse = |text: &str| -> Result<(usize, Option<usize>, Option<usize>), ()> {
        let mut nums = text.split('/').map(|n| n.parse::<isize>());
        let v = index_to_unsigned(nums.next().unwrap().unwrap(), cur_v_len);
        let vt = nums.next().map(|idx| index_to_unsigned(idx.unwrap(), cur_vn_len));
        let vn = nums.next().map(|idx| index_to_unsigned(idx.unwrap(), cur_vt_len));
        Ok((v, vt, vn))
    };

    // Split on whitespace
    let mut verts = text.trim().split_whitespace();

    // Parse the first one first

    todo!();
}

fn read_line_elem(text: &str, data: &ObjData) -> LineElement {
    todo!()
}

fn read_face_elem(text: &str, data: &ObjData) -> FaceElement {
    todo!()
}


// fn read_slashed_indices(text: &str) -> Result<Vec<usize>> {}

// fn parse_face(s: &str) -> Result<Vec<FaceVertex>, FaceError> {
//     fn parse_face_vertex(s: &str) -> Result<FaceVertex, FaceError> {
//         let mut nums = s
//             .split('/')
//             .into_iter()
//             .map(|txt| txt.parse::<NonZeroIsize>().or(Err(FaceError::InvalidVertex)));

//         Ok(FaceVertex {
//             v: nums.next().ok_or(FaceError::InvalidVertex)??,
//             vt: nums.next().transpose()?,
//             vn: nums.next().transpose()?,
//         })
//     }

//     // Spec says there's no space between the slashes, so we can just split on whitespace again
//     let mut vertices = s.trim().split_whitespace();

//     // Parse the first one to see how the slashes are done
//     let v1 = parse_face_vertex(vertices.next().ok_or(FaceError::NotEnoughVertices)?)?;

//     // Now parse the rest and ensure they match
//     vertices
//         .into_iter()
//         .map(|txt| {
//             parse_face_vertex(txt).and_then(|v| {
//                 if v.vn.is_some() != v1.vn.is_some() || v.vt.is_some() != v1.vt.is_some() {
//                     Err(FaceError::MismatchedSlashes)
//                 } else {
//                     Ok(v)
//                 }
//             })
//         })
//         .collect()
// }

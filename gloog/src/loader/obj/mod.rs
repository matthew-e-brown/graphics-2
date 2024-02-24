//! Module for loading models from OBJ files, optionally paired with MTL material files.
//!
//! Only all-polygonal models are supported are the moment (no free-form surfaces). These are those that use only `f`,
//! no `curv` or `surf` statements. Unsupported statements are simply ignored, though a warning is produced.
//!
//! For future reference, here are some of the documents I used for the OBJ and MTL specs:
//!
//! - https://www.uhu.es/francisco.moreno/gii_rv/practicas/practica08/OBJ_SPEC.PDF
//! - also: https://paulbourke.net/dataformats/obj/ (missing math section)
//! - https://paulbourke.net/dataformats/mtl/

mod error;
mod mtl;

use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::num::{NonZeroUsize, ParseFloatError};
use std::path::Path;
use std::sync::Arc;

use arrayvec::ArrayVec;
use gloog_core::bindings::types::GLuint;
use gloog_math::{Vec2, Vec3};
use log::{debug, log, trace, warn};

use self::error::ObjLoadError;
use crate::loader::{fmt_line_range, lines_escaped, LineRange};


// cspell:words curv interp stech ctech scrv cstype bmat usemtl mtllib maplib usemap


/// The constant that is used for primitive restarting in OpenGL.
const PRIMITIVE_RESTART: GLuint = GLuint::MAX;


/// A drawable model comprised of data from an OBJ file.
///
/// The raw data behind this struct are behind [`Arc`] pointers, so this struct can be cloned cheaply without the need
/// for reallocating or re-parsing an entire model.
#[derive(Debug, Clone)]
pub struct ObjModel {
    /// Individual groups of faces which share a common material (and thus will share a single draw call). Each group
    /// contains a buffer of indices which index into [`Self::data`] using `glDrawElements`.
    groups: Arc<[ObjGroup]>,
    /// All raw vertex attribute data for this model. This data is referenced through indices in [`Self::groups`].
    data: Arc<[ObjVertex]>,
}

/// Holds a group of faces to be drawn with a common material.
///
/// Comprised of material information and a list of indices
#[derive(Debug)]
pub struct ObjGroup {
    /// Contains information for how to set uniform information for this group's draw call. for this particular group of
    /// faces. A value of `None` means that the "default" material should be used, whatever it may be.
    pub material: Option<ObjMaterial>,
    /// An array of indices to be drawn with `glDrawElements`, which indexes into the parent [`ObjModel`]'s data.
    indices: Box<[GLuint]>,
}

/// Used to configure uniforms before executing draw call.
#[derive(Debug, Default, Clone)]
pub struct ObjMaterial {
    // todo: texture maps
    pub diffuse: Vec3,
    pub ambient: Vec3,
    pub specular: Vec3,
    pub spec_pow: f32,
    /// Parsed from the "dissolve" statement, which relates to some more advanced material and illumination properties;
    /// we only support basic Blinn-Phong, so we're just gonna interpret it as an alpha channel for the final color.
    pub opacity: f32,
}

#[repr(C)]
#[derive(Debug, Clone)]
struct ObjVertex {
    position: Vec3,
    tex_coord: Vec2,
    normal: Vec3,
}


/// A triple of indices into the three different sets of vertex data. Indices are 1-based to allow the optional values
/// to represent `None` using zero.
type FaceIndices = (NonZeroUsize, Option<NonZeroUsize>, Option<NonZeroUsize>);


impl ObjModel {
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self, ObjLoadError> {
        let path = path.as_ref();
        let file = BufReader::new(File::open(path)?);

        // Phase 1: Gathering data
        // --------------------------------------------------------------------

        let mut v_data = Vec::new(); // Vertex positions from `v` statements
        let mut vt_data = Vec::new(); // Texture coordinates from `vt` statements
        let mut vn_data = Vec::new(); // Vertex normals from `vn` statements

        let mut parsed_materials = Vec::<ObjMaterial>::new(); // List of all materials
        let mut material_indices = HashMap::<Box<str>, usize>::new(); // Map of text name to index of material in list
        let mut curr_material = None; // Index of currently active material

        // List of faces and which materials they use
        let mut face_idx_buffer = Vec::new(); // List of all of the `FaceIndices` (each makes one final vertex)
        let mut face_vert_counts = Vec::new(); // Index `i` says how many verts face `i` uses
        let mut face_material_map = Vec::new(); // Index `i` names which material face `i` uses (as an index).

        for line_result in lines_escaped(file) {
            let (line_nums, line) = line_result?;

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
            // zero-length, and we did so after trimming, which guarantees that there is at least one thing
            // pre-whitespace).
            let directive = line.split_whitespace().nth(0).unwrap();
            let line = line[directive.len()..].trim_start(); // rest of the line

            match directive {
                "v" => v_data.push(parse_v(line, &line_nums)?),
                "vt" => vt_data.push(parse_vt(line, &line_nums)?),
                "vn" => vn_data.push(parse_vn(line, &line_nums)?),
                "f" => {
                    // Parse all the vertex reference numbers on this line into indices into our vertex data. All are
                    let cur_sizes = (v_data.len(), vt_data.len(), vn_data.len());
                    let vert_iter = parse_f(line, cur_sizes, &line_nums);

                    // How many vertices we have before push
                    let before_push = face_idx_buffer.len();

                    // Reserve before our loop (`saturating_add(1)` borrowed from Vec's `extend` implementation).
                    face_idx_buffer.reserve(vert_iter.size_hint().0.saturating_add(1));
                    for vertices in vert_iter {
                        face_idx_buffer.push(vertices?);
                    }

                    // Check that we pushed at least three vertices for this face, also double check we don't have too
                    // many elements for this face (since we're using a u16 to reduce the footprint of this function).
                    let pushed = face_idx_buffer.len() - before_push;
                    if pushed < 3 {
                        return Err(ObjLoadError::f_too_few(&line_nums, pushed));
                    } else if pushed > u16::MAX as usize {
                        return Err(ObjLoadError::f_too_many(&line_nums, pushed));
                    }

                    // Trim the count down to smaller number type for memory savings; also save the material that this
                    // face used.
                    face_vert_counts.push(pushed as u16);
                    face_material_map.push(curr_material);
                },
                "usemtl" => {
                    // - Check the name-to-index map, update current material if found
                    // - Otherwise... crash. lol
                    warn!("(unimplemented) 'usemtl' directive on {}", fmt_line_range(&line_nums));
                },
                "mtllib" => {
                    // Parse mtl file:
                    // - For each material, check if its name already appears in the index map
                    // - If not, parse it and add it to the list, and add its index to the map
                    // - Don't forget that `mtllib` statements may have multiple filenames specified
                    warn!("(unimplemented) 'mtllib' directive on {}", fmt_line_range(&line_nums));
                },
                _ => check_other(directive, &line_nums)?,
            }
        }

        // There should be one of each of these per face.
        assert_eq!(face_vert_counts.len(), face_material_map.len());

        // Phase 2: Merging it all together
        // --------------------------------------------------------------------

        // For each face, look at its `FaceIndex` (triple of position, texture, normal data) and grab the required data
        // from the full list of vertex data. If that particular triple has been encountered before, we can re-use that
        // index. We have to do this merging by `v/vt/vn` triple instead of per each of the constituent components
        // because OpenGL does not support using different indices for each attribute of a vertex.

        let mut vertex_data = Vec::new(); // Final vertex attributes after merging through face indices
        let mut vertex_map = HashMap::new(); // Lookup table for whether or not a triple has been included yet

        // List of final indices into final vertex data, grouped by material (using the index of the material).
        // Individual faces are separated only by the `PRIMITIVE_RESTART_INDEX`.
        let mut groups: HashMap<Option<usize>, Vec<GLuint>> = HashMap::new();

        for (vert_count, material_idx) in face_vert_counts.into_iter().zip(face_material_map.into_iter()) {
            // This loop runs once per face
            // ----------------------------------------------------------------

            let vert_count = vert_count as usize; // back to usize for indexing
            let vert_indices = &face_idx_buffer[..vert_count];

            // Ensure we can push this many more vertices into our main buffer and still be able to index into it with
            // `GL_UNSIGNED_INT`:
            if vertex_data.len() + vert_count >= GLuint::MAX as usize {
                return Err(ObjLoadError::VertexDataOverflow);
            }

            // We know from our `pushed < 3` check earlier that each section of vertices is *at least* three. So before
            // we loop over all the vertices of this face, check if we need to compute the surface normal (to use for
            // all vertex normals) for this face using the first three (NB: tuple order is `v/vt/vn`).
            let surf_norm = if vert_indices[0].2.is_none() {
                // This'll break if a face defines more than 3 vertices that aren't coplanar; but that's not exactly my
                // fault.
                let a = v_data[vert_indices[0].0.get() - 1];
                let b = v_data[vert_indices[1].0.get() - 1];
                let c = v_data[vert_indices[2].0.get() - 1];
                let ab = b - a;
                let ac = c - a;
                Some(ab.cross(&ac))
            } else {
                None
            };

            // Grab the list that this face is going to push its indices into
            let index_list = groups.entry(material_idx).or_default();

            // Now that we've done some double checking on the vertices for this face, drain them from the vector.
            for vert_indices in face_idx_buffer.drain(..vert_count) {
                // For each vertex in this face, check if we've already added its index triple to the final buffer, and
                // push it into this group's list of indices. If not, go grab the necessary data, insert into the vertex
                // buffer, and push the new index.
                index_list.push(*vertex_map.entry(vert_indices).or_insert_with(|| {
                    // Don't forget to undo the 1-based indexing before actually doing our lookups.
                    let (v_idx, vt_idx, vn_idx) = vert_indices;

                    let v = v_data[v_idx.get() - 1]; // always present
                    let vt = vt_idx.map(|i| vt_data[i.get() - 1]).unwrap_or_default(); // just use a (0,0) vector
                    let vn = vn_idx.map(|i| vn_data[i.get() - 1]).or(surf_norm).unwrap(); // at least one will be Some

                    vertex_data.push(ObjVertex {
                        position: v,
                        tex_coord: vt,
                        normal: vn,
                    });

                    // Already did a bounds-check earlier so we know this vertex's index won't overflow a `u32`
                    (vertex_data.len() - 1) as GLuint
                }));
            }

            // End of our face, so we want to restart our primitive rendering, too.
            index_list.push(PRIMITIVE_RESTART);
        }

        // I don't even wanna think about how much heap allocation there is in this function... oh well, it's probably
        // still less than there'd be in something like JavaScript lol.

        // Convert our indices into boxed slices, and finally grab the actual material values.
        Ok(ObjModel {
            data: vertex_data.into(),
            groups: groups
                .into_iter()
                .map(|(material_idx, indices)| ObjGroup {
                    indices: indices.into_boxed_slice(),
                    material: material_idx.map(|i| parsed_materials[i].clone()),
                })
                .collect(),
        })
    }
}


/// Reads at most `N` whitespace-separated floats from a line of text, returning them alongside the number that remain
/// on the line.
fn read_ws_verts<const N: usize>(text: &str) -> Result<(ArrayVec<f32, N>, usize), ParseFloatError> {
    // Take at most `N` vertices
    let mut pieces = text.split_whitespace().map(|s| s.parse());
    let floats = pieces.by_ref().take(N).collect::<Result<ArrayVec<f32, N>, _>>()?;
    let remaining = pieces.count();
    Ok((floats, remaining))
}

/// Parses the body of a `v` directive from an OBJ file and inserts its resultant vector into the given list. `lines` is
/// needed for error-reporting.
fn parse_v(text: &str, lines: &LineRange) -> Result<Vec3, ObjLoadError> {
    // Take at most four; some files seem to specify extra `1` values after the w to force some viewers to get the
    // message (that's my guess anyways).
    let Ok((floats, remaining)) = read_ws_verts::<4>(text) else {
        return Err(ObjLoadError::v_parse_err(lines, "v"));
    };

    // xyz are required; w is optional and defaults to 1. In OBJ, w is only used for free-form curves and surfaces:
    // there are no homogeneous coordinates/transformations within OBJ files; everything is simply stored in 3D space.
    // Meaning, we should never run into a case where `w` it isn't 1. If we do, simply emit a small warning/notice that
    // the file _may_ contain unsupported features and that we're ignoring it.
    if floats.len() < 3 {
        Err(ObjLoadError::v_too_small(lines, "v", floats.len(), 3))
    } else if remaining > 0 {
        Err(ObjLoadError::v_too_large(lines, "v", floats.len() + remaining, 4))
    } else {
        if floats.len() == 4 && floats[3] != 1.0 {
            debug!(
                "ignoring non-1.0 'w' component of '{}', since 'w' only affects free-form geometry (unsupported)",
                text
            );
        }

        // Take the first three and convert them into an array, then into a vector; we know there're at least 3.
        let vertex = [floats[0], floats[1], floats[2]].into();
        trace!("parsed vertex {vertex:?}");
        Ok(vertex)
    }
}

/// Parses the body of a `vn` directive from an OBJ file.
fn parse_vn(text: &str, lines: &LineRange) -> Result<Vec3, ObjLoadError> {
    // Take as many as they provided, in a Vec. The spec says that there should always be three.
    let Ok((floats, remaining)) = read_ws_verts::<3>(text) else {
        return Err(ObjLoadError::v_parse_err(lines, "vn"));
    };

    if floats.len() < 3 {
        Err(ObjLoadError::v_too_small(lines, "vn", floats.len(), 3))
    } else if remaining > 0 {
        Err(ObjLoadError::v_too_large(lines, "vn", floats.len() + remaining, 3))
    } else {
        // We know there's exactly three now
        let normal = [floats[0], floats[1], floats[2]].into();
        trace!("parsed normal {normal:?}");
        Ok(normal)
    }
}

/// Parses the body of a `vt` directive from an OBJ file.
fn parse_vt(text: &str, lines: &LineRange) -> Result<Vec2, ObjLoadError> {
    // Take at most three
    let Ok((floats, remaining)) = read_ws_verts::<3>(text) else {
        return Err(ObjLoadError::v_parse_err(lines, "vt"));
    };

    let tc = match floats.len() {
        0 => return Err(ObjLoadError::v_too_small(lines, "vt", floats.len(), 1)),
        1 => [floats[0], 0.0].into(),
        2 => [floats[0], floats[1]].into(),
        3 if remaining == 0 => {
            debug!("found texture coord with 3rd dimension; 3rd dimension will be ignored");
            [floats[0], floats[1]].into()
        },
        n => return Err(ObjLoadError::v_too_large(lines, "vt", n + remaining, 3)),
    };

    trace!("parsed tex-coord {tc:?}");

    Ok(tc)
}


/// Parses the body of an `f` directive from an OBJ file.
fn parse_f<'a>(
    text: &'a str,
    current_counts: (usize, usize, usize),
    lines: &'a LineRange,
) -> impl Iterator<Item = Result<FaceIndices, ObjLoadError>> + 'a {
    /// Each face is comprised of a list of indices into vertex data, each one specifying one vertex in the face. It may
    /// be comprised of just position data, but may also include indices into texture and normal-data.
    ///
    /// Each element in the list must be of the same form, so we need to track which format this specific face's
    /// vertices use.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum RefShape {
        Single, // `v`
        Double, // `v/vt`
        Triple, // `v/vt/vn`
        NoTex,  // `v//vn`
    }

    impl RefShape {
        pub fn check<T>(vt: &Option<T>, vn: &Option<T>) -> Self {
            match (vt, vn) {
                (None, None) => Self::Single,
                (Some(_), None) => Self::Double,
                (Some(_), Some(_)) => Self::Triple,
                (None, Some(_)) => Self::NoTex,
            }
        }
    }

    let mut shape = None;

    // For each `(v/vt/vn) (v/vt/vn) (v/vt/vn)` in the line...
    text.split_whitespace().map(move |v_refs_str| {
        // Parses each of the slash-separated "vertex references" into a `usize`, taking into account any
        // negative/relative indexing. `vec_len` and `vec_name` are used for error-checking.
        let parse_v_ref = |ref_str: &str, vec_len: usize, vec_name: &'static str| {
            match ref_str.parse::<isize>() {
                // Positive: check for overflow.
                Ok(i) if i > 0 => match usize::try_from(i).unwrap() {
                    i if i - 1 < vec_len => Ok(NonZeroUsize::new(i).unwrap()),
                    _ => Err(ObjLoadError::f_index_range(lines, vec_name, i, vec_len)),
                },
                // Negative: subtract it from list size, check for overflow, and add one to return to 1-based index.
                Ok(i) if i < 0 => match vec_len.checked_sub((-i).try_into().unwrap()) {
                    Some(i) => Ok(NonZeroUsize::new(i + 1).unwrap()),
                    None => Err(ObjLoadError::f_index_range(lines, vec_name, i, vec_len)),
                },
                // Zero: out of range.
                Ok(i) => Err(ObjLoadError::f_index_range(lines, vec_name, i, vec_len)),
                Err(_) => Err(ObjLoadError::f_parse_err(lines)),
            }
        };

        // Split on slashes to get each component:
        let mut indices = v_refs_str.split('/');

        // Then parse once for each of `v`, `vt`, and `vn`:
        let (v_len, vt_len, vn_len) = current_counts;

        // There must be at least one item (splitting "v" on '/' will just yield "v"), so we are safe to unwrap the
        // first thing; error-check with `?` after doing so. `vt` and `vn` will be `None` if there weren't enough
        // slashes; `Option<&str> -> Option<Result<usize>>`, transpose to `Result<Option<usize>>` and `?`.
        let v = parse_v_ref(indices.next().unwrap(), v_len, "v")?;
        let vt = indices.next().map(|s| parse_v_ref(s, vt_len, "vt")).transpose()?;
        let vn = indices.next().map(|s| parse_v_ref(s, vn_len, "vn")).transpose()?;

        // Double check that this triple of reference numbers matches the first one. If this is the first one, store
        // what shape this one had.
        let this_shape = RefShape::check(&vt, &vn);
        match shape {
            None => shape = Some(this_shape),
            Some(shape) if this_shape != shape => return Err(ObjLoadError::f_mismatched(lines)),
            Some(_) => (), // otherwise, we can just continue onwards
        }

        // There should only be at most three slash-separated components; if there are more remaining after parsing the
        // first three, we have a problem.
        if indices.count() > 0 {
            Err(ObjLoadError::f_parse_err(lines))
        } else {
            Ok((v, vt, vn))
        }
    })
}


/// Checks whether or not an unsupported directive can be skipped gracefully and logs it if applicable.
pub(crate) fn check_other(directive: &str, lines: &LineRange) -> Result<(), ObjLoadError> {
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
        _ => return ObjLoadError::unknown(lines, directive).into(),
    };

    log!(level, "ignoring *.obj directive '{directive}'; feature unsupported");
    Ok(())
}

mod mtl;

use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::num::{NonZeroU16, NonZeroUsize, ParseFloatError};
use std::path::Path;
use std::sync::Arc;

use arrayvec::ArrayVec;
use gloog_core::bindings::types::GLuint;
use gloog_math::{Vec2, Vec3};
use log::{debug, log, trace, warn};

use super::error::ObjParseError;
use crate::loader::{fmt_line_range, lines_escaped, LineRange};


/// The constant that is used for primitive restarting in OpenGL.
const PRIMITIVE_RESTART: GLuint = GLuint::MAX;


// cspell:words curv interp stech ctech scrv cstype bmat usemtl mtllib maplib usemap

// =====================================================================================================================
// Thought: Arc<RawData> which holds several boxes, or. just holding several Arcs?
//
//  ->  Consideration: probably best to hold multiple Arcs. Arc->Box->Data is a single ref-count and increment when
//      cloning, but is a double pointer indirection. With (Arc->Data)+(Arc->Data)+..., we only have a single
//      indirection on each access. Accessing is probably more common than cloning, so we'll stick with that.
//
// Extra rambling/notes: Sadly, there's no way to do custom DSTs in Rust yet. What I'd prefer to do: leave `groups` and
// `buffer` as totally unsized `[slices]`, then implement [`std::ptr::Pointee`] and mark that two `usize`s are needed
// for pointer metadata -- one to know how far into the struct to find the end of each field. Sadly, there isn't
// currently a way to define custom DSTs like this. There also doesn't even seem to be a debate yet on a way to tell
// Rust _how_ your Pointee metadata should be used; i.e, even if I could implement `Pointee::Metadata` as `(usize,
// usize)`, there's no way to explain to the compiler that the first one is the length of the first field and that the
// second is the length of the second field.
//
// I *could* do that manually by simply allocating a giant buffer of bytes to my exact known size, then provide getter
// functions that use `unsafe` to transmute ranges of that buffer into "fields"... But that seems like a lotta work and
// a lotta `unsafe` just to avoid a few extra pointer indirections.
// =====================================================================================================================


/// A drawable model comprised of data from an OBJ file.
///
/// The raw data behind this struct are behind [`Arc`] pointers, so this struct can be cloned cheaply without the need
/// for reallocating or re-parsing an entire model.
#[derive(Debug, Clone)]
pub struct ObjModel {
    /// Individual groups of faces which share a common material (and thus will share a single draw call). Each group
    /// contains a buffer of indices which index into [`Self::vertex_data`] using `glDrawElements`.
    elements: Arc<[ObjGroup]>,
    /// All raw vertex attribute data for this model. This data is referenced through indices in [`Self::elements`].
    vertex_data: Arc<[ObjVertex]>,
}

/// Specifies a series of elements to be drawn with a common material.
///
/// For example:
///
/// ```
/// indices = [[8, 9, 1, 2], [8, 1, 2], [0, 4, 5, 8, 1, 3]];
/// counts = [4, 3, 6];
/// ```
///
/// This setup will draw one combined polygon with vertices 8, 9, 1, and 2; one with vertices 8, 1, and 2, and one with
/// 0, 4, 5, 8, 1, and 3.
#[derive(Debug)]
struct ObjGroup {
    /// Contains information for how to set uniform information for this group's draw call. for this particular group of
    /// faces. A value of `None` means that the "default" material should be used, whatever it may be.
    material: Option<ObjMaterial>,
    /// A 2D array, specifically with extra pointer indirection to be compatible with `glMultiDrawElements`'s parameter
    /// of `const void * const *`.
    indices: Box<[GLuint]>,
}

/// Used to configure uniforms before executing draw call.
#[derive(Debug, Default, Clone)]
pub struct ObjMaterial {
    pub ka: Vec3,
    pub kd: Vec3,
    pub ks: Vec3,
    pub ns: f32,
    pub opacity: f32,
    // todo: should these be options instead of just letting them be defaulted to zeroes?
    // todo: texture maps
    // todo: any other missing fields from MTL spec?
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
    pub fn load_from_file(path: impl AsRef<Path>) -> Result<Self, ObjParseError> {
        let path = path.as_ref();
        let file = BufReader::new(File::open(path)?);

        // Phase 1: Gathering data
        // --------------------------------------------------------------------

        let mut v_data = Vec::new(); // Vertex positions from `v` statements
        let mut vt_data = Vec::new(); // Texture coordinates from `vt` statements
        let mut vn_data = Vec::new(); // Vertex normals from `vn` statements

        let mut parsed_materials: Vec<ObjMaterial> = Vec::new(); // List of all materials
        let mut material_indices: HashMap<Box<str>, NonZeroU16> = HashMap::new(); // material name -> index in vector
        let mut curr_material: Option<NonZeroU16> = None; // Index of currently active material

        // List of faces and which materials they use
        let mut index_buffer = Vec::new(); // List of all of the FaceIndices (each makes one final vertex)
        let mut face_vert_counts = Vec::new(); // Index `i` says how many verts face `i` uses
        let mut face_material_map = Vec::new(); // Index `i` names which material face `i` uses (as an index).

        // ok... I've got a big-ass list of indices that I'm indexing by storing another list of indices that index into
        // that list of indices (which index into my other big-ass lists of vertex data). Totally not confusing... Let's
        // just call the indices into vertex data "vertices," that'll make things easier.

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
            let rest = line[directive.len()..].trim_start(); // rest of the line

            match directive {
                "v" => v_data.push(parse_v(rest, &line_nums)?),
                "vt" => vt_data.push(parse_vt(rest, &line_nums)?),
                "vn" => vn_data.push(parse_vn(rest, &line_nums)?),
                "f" => {
                    // Parse all the vertex ref numbers on this line into all-positive indices into our vertex data. For
                    // each one,
                    let cur_sizes = (v_data.len(), vt_data.len(), vn_data.len());
                    let vert_iter = parse_f(rest, cur_sizes, &line_nums);

                    // How many vertices we have before push
                    let before_push = index_buffer.len();

                    // Reserve before our loop (`saturating_add(1)` borrowed from Vec's `extend` implementation).
                    let (hint, _) = vert_iter.size_hint();
                    index_buffer.reserve(hint.saturating_add(1));
                    for vertices in vert_iter {
                        index_buffer.push(vertices?);
                    }

                    // Check that we pushed at least three vertices for this face
                    let pushed = index_buffer.len() - before_push;
                    if pushed < 3 {
                        return Err(ObjParseError::f_too_few(&line_nums, pushed));
                    } else if pushed > u16::MAX as usize {
                        return Err(ObjParseError::f_too_many(&line_nums, pushed));
                    }

                    // Trim the count down to smaller number type for memory savings; also save the material that this
                    // face used.
                    face_vert_counts.push(pushed as u16);
                    face_material_map.push(curr_material);
                },
                "usemtl" => {
                    //
                    warn!("(unimplemented) 'usemtl' directive on {}", fmt_line_range(&line_nums));
                },
                "mtllib" => {
                    // Parse mtl file.
                    // - For each material, check if its name has already been used in `material_indices` map.
                    // - If not, add info to `materials` list then add its index to `material_indices`.
                    warn!("(unimplemented) 'mtllib' directive on {}", fmt_line_range(&line_nums));
                },
                _ => check_other(directive, &line_nums)?,
            }
        }

        // There should be one of each of these per face.
        assert_eq!(face_vert_counts.len(), face_material_map.len());

        // Phase 2: Merge it all together.
        // --------------------------------------------------------------------
        // Time to index the shit outta these vectors, homie.

        let mut final_vertex_data = Vec::new(); // all vertex attributes, after they've been merged through indexing
        let mut final_vertex_map = HashMap::new(); // where to find merged-attributes based on the triples of indices

        // Final map of materials -> list of indices before we merge them into a list of `ObjGroup` structs
        let mut mtl_idx_groups: HashMap<Option<NonZeroU16>, Vec<GLuint>> = HashMap::new();

        for (vert_count, material_index) in face_vert_counts.into_iter().zip(face_material_map.into_iter()) {
            // This loop runs once per face
            // ----------------------------------------------------------------

            let vert_count = vert_count as usize; // back to usize for indexing
            let vert_indices = &index_buffer[..vert_count];

            // Ensure we can push this many more vertices into our main buffer and still be able to index into it with
            // `GL_UNSIGNED_INT`:
            if final_vertex_data.len() + vert_count >= GLuint::MAX as usize {
                return Err(ObjParseError::VertexDataOverflow);
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
            let index_list = mtl_idx_groups.entry(material_index).or_default();

            // Now that we've done some double checking on the vertices for this face, drain them from the vector. For
            // each vertex in this face, check if we've already added it (ie. this particular combo of indices) to the
            // final buffer. We can re-used it if we have.
            for vert_indices in index_buffer.drain(..vert_count) {
                // Try and locate its index, and if we haven't added it before, insert our vertex data and
                let v_data_idx = *final_vertex_map.entry(vert_indices).or_insert_with(|| {
                    // Don't forget to undo our 1-based vertex indices. At least one of `face_normal` (from earlier
                    // check) or the data from the vector will be `Some` here.
                    let (v_idx, vt_idx, vn_idx) = vert_indices;
                    let v = v_data[v_idx.get() - 1];
                    let vt = vt_idx.map(|i| vt_data[i.get() - 1]).unwrap_or_default();
                    let vn = surf_norm.or(vn_idx.map(|i| vn_data[i.get() - 1])).unwrap();

                    // Push into the list and return the new spot
                    final_vertex_data.push(ObjVertex {
                        position: v,
                        tex_coord: vt,
                        normal: vn,
                    });

                    // Already did a bounds-check earlier, so we know this index of our freshly pushed vertex won't
                    // overflow a `u32`.
                    (final_vertex_data.len() - 1) as GLuint
                });

                // This index is what gets added to this group.
                index_list.push(v_data_idx);
            }

            // End of our face, so we want to restart our primitive rendering, too.
            index_list.push(PRIMITIVE_RESTART);
        }

        // I don't even wanna think about how much heap allocation there is in this function... oh well, it's probably
        // still less than there'd be in JavaScript, lol.

        // Convert our indices into boxed slices, and finally grab the actual material values.
        Ok(ObjModel {
            vertex_data: final_vertex_data.into(),
            elements: mtl_idx_groups
                .into_iter()
                .map(|(mtl_idx, all_indices)| ObjGroup {
                    indices: all_indices.into_boxed_slice(),
                    material: mtl_idx.map(|i| parsed_materials[(i.get() - 1) as usize].clone()),
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
fn parse_v(text: &str, lines: &LineRange) -> Result<Vec3, ObjParseError> {
    // Take at most four; some files seem to specify extra `1` values after the w to force some viewers to get the
    // message (that's my guess anyways).
    let Ok((floats, remaining)) = read_ws_verts::<4>(text) else {
        return Err(ObjParseError::v_parse_err(lines, "v"));
    };

    // xyz are required; w is optional and defaults to 1. In OBJ, w is only used for free-form curves and surfaces:
    // there are no homogeneous coordinates/transformations within OBJ files; everything is simply stored in 3D space.
    // Meaning, we should never run into a case where `w` it isn't 1. If we do, simply emit a small warning/notice that
    // the file _may_ contain unsupported features and that we're ignoring it.
    if floats.len() < 3 {
        Err(ObjParseError::v_too_small(lines, "v", floats.len(), 3))
    } else if remaining > 0 {
        Err(ObjParseError::v_too_large(lines, "v", floats.len() + remaining, 4))
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
fn parse_vn(text: &str, lines: &LineRange) -> Result<Vec3, ObjParseError> {
    // Take as many as they provided, in a Vec. The spec says that there should always be three.
    let Ok((floats, remaining)) = read_ws_verts::<3>(text) else {
        return Err(ObjParseError::v_parse_err(lines, "vn"));
    };

    if floats.len() < 3 {
        Err(ObjParseError::v_too_small(lines, "vn", floats.len(), 3))
    } else if remaining > 0 {
        Err(ObjParseError::v_too_large(lines, "vn", floats.len() + remaining, 3))
    } else {
        // We know there's exactly three now
        let normal = [floats[0], floats[1], floats[2]].into();
        trace!("parsed normal {normal:?}");
        Ok(normal)
    }
}

/// Parses the body of a `vt` directive from an OBJ file.
fn parse_vt(text: &str, lines: &LineRange) -> Result<Vec2, ObjParseError> {
    // Take at most three
    let Ok((floats, remaining)) = read_ws_verts::<3>(text) else {
        return Err(ObjParseError::v_parse_err(lines, "vt"));
    };

    let tc = match floats.len() {
        0 => return Err(ObjParseError::v_too_small(lines, "vt", floats.len(), 1)),
        1 => [floats[0], 0.0].into(),
        2 => [floats[0], floats[1]].into(),
        3 if remaining == 0 => {
            debug!("found texture coord with 3rd dimension; 3rd dimension will be ignored");
            [floats[0], floats[1]].into()
        },
        n => return Err(ObjParseError::v_too_large(lines, "vt", n + remaining, 3)),
    };

    trace!("parsed tex-coord {tc:?}");

    Ok(tc)
}


/// Parses the body of an `f` directive from an OBJ file.
fn parse_f<'a>(
    text: &'a str,
    current_counts: (usize, usize, usize),
    lines: &'a LineRange,
) -> impl Iterator<Item = Result<FaceIndices, ObjParseError>> + 'a {
    /// Each face is a list of indices into vertex data. Indices may be of the form `v`, `v/vt`, `v/vt/vn`, or `v//vn`.
    ///
    /// Each element in the list must be of the same form. Additionally, indices are 1-based; they may also be negative.
    /// Negative indices represent an index relative to the _current_ end of the list (ie., the most recently parsed
    /// vertex). There must be at least three vertices for each face.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum RefShape {
        /// `v` - This face contains just `v` references.
        Single,
        /// `v/vt` - This face contains `v` and `vt` references separated by a slash.
        Double,
        /// `v/vt/vn` - This face contains `v`, `vt`, and `vn` references separated by two slashes.
        Triple,
        /// `v//vn` - This face contains `v` and `vn` references, with nothing between two slashes (where the `vt`
        /// reference would usually go).
        NoTex,
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
                // If it's positive, all we have to do is ensure it doesn't overflow our list.
                Ok(i) if i > 0 => match usize::try_from(i).unwrap() {
                    i if i - 1 < vec_len => Ok(NonZeroUsize::new(i).unwrap()),
                    _ => Err(ObjParseError::f_index_range(lines, vec_name, i, vec_len)),
                },
                // If it's negative, subtract it from the list size to get 0-based index; ensure it won't underflow.
                Ok(i) if i < 0 => match vec_len.checked_sub((-i).try_into().unwrap()) {
                    Some(i) => Ok(NonZeroUsize::new(i + 1).unwrap()), // Add 1 to return to 1-based for consistency.
                    None => Err(ObjParseError::f_index_range(lines, vec_name, i, vec_len)),
                },
                // If it's neither, it's zero, and out of range.
                Ok(i) => Err(ObjParseError::f_index_range(lines, vec_name, i, vec_len)),
                Err(_) => Err(ObjParseError::f_parse_err(lines)),
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
            Some(shape) if this_shape != shape => return Err(ObjParseError::f_mismatched(lines)),
            Some(_) => (), // otherwise, we can just continue onwards
        }

        // There should only be at most three slash-separated components; if there are more remaining after parsing the
        // first three, we have a problem.
        if indices.count() > 0 {
            Err(ObjParseError::f_parse_err(lines))
        } else {
            Ok((v, vt, vn))
        }
    })
}

/// Checks whether or not an unsupported directive can be skipped gracefully and logs it if applicable.
pub(crate) fn check_other(directive: &str, lines: &LineRange) -> Result<(), ObjParseError> {
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
        _ => return ObjParseError::unknown(lines, directive).into(),
    };

    log!(level, "ignoring *.obj directive '{directive}'; feature unsupported");
    Ok(())
}

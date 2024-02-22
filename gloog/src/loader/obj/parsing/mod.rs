mod mtl;

use std::fs::File;
use std::io::BufReader;
use std::num::ParseFloatError;
use std::path::Path;
use std::sync::Arc;

use arrayvec::ArrayVec;
use gloog_core::bindings::types::GLuint;
use gloog_math::{Vec2, Vec3};
use log::{debug, log, trace, warn};

use super::error::ObjParseError;
use crate::loader::{fmt_line_range, lines_escaped, LineRange};


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


///
#[derive(Debug, Clone)]
pub struct ObjModel {
    /// Individual groups of faces which share a common material (and thus will share a single draw call). Each group
    /// contains a buffer of indices with which to index into [`Self::raw_vbuffer`].
    groups: Arc<[ObjGroup]>,
    /// Stored on the heap as well since (future versions of) materials may also hold texture data.
    materials: Arc<[ObjMaterial]>,
    /// All raw vertex attribute data for this model. This data is referenced through indices in [`Self::elements`].
    buffer: Arc<[u8]>,
}

#[derive(Debug)]
struct ObjGroup {
    /// Which material to use (as an index into [`ObjModel::materials`]) for this particular group of faces.
    material: usize,
    /// An OpenGL _element array buffer_ store for drawing this group of faces.
    indices: Box<[GLuint]>,
}

/// Used to configure uniforms before executing draw call.
#[derive(Debug, Default, Clone)]
pub struct ObjMaterial {
    // todo: should these stay as options, or be replaced with defaults upon parse?
    pub ka: Option<Vec3>,
    pub kd: Option<Vec3>,
    pub ks: Option<Vec3>,
    pub ns: Option<f32>,
    pub opacity: Option<f32>,
    // todo: maps
    // todo: any other missing fields from MTL spec?
}


impl ObjModel {
    pub fn load_from_file(path: impl AsRef<Path>) -> Result<Self, ObjParseError> {
        let path = path.as_ref();
        let file = BufReader::new(File::open(path)?);

        // - Read all vertices
        // - Read all faces
        // - Each unique combination of vertex references (eg. `a/b/c`, `a/b/d` are two separate ones) constitutes one
        //   "vertex" in the final output buffer (since it's not possible to use different indices for multiple
        //   attributes in OpenGL).
        // - So, generate our own hash-set of `usize/Option<usize>/Option<usize>`; these are then used to index into our
        //   raw data at the end.
        //     - TODO: maybe keep them as 1-based indices to take advantage of `Option<NonZeroUSize>` space
        //       optimization?)
        // - While looping through faces, keep track of all the faces as collections of indices into our new set of
        //   un-tangled vertices
        //     - When doing said tracking, bin them based on the `g` group they're a part of and which material they
        //       use. That will allow the `g` groups in the file to have some control over draw-call ordering.
        // - Then at the end, run through our mapping and copy over the relevant data from our raw lists into our final
        //   buffers.
        //     - This'll sadly double (!!) our space complexity during that final parse, but we kinda have to do it this
        //       way, since we have no way to know what our final strides/offsets should be until all the dust settles
        //       with our face indices.
        // - Also at the end, run through each of our bins of faces and merge each bin into one long list of indices.

        // PROBLEM: one long list of indices will generate one long triangle fan. unless I feel like triangulating all
        // this stuff myself. POTENTIAL SOLUTION: look into `glMultiDrawElements`. Basically, make one `glDrawElements`
        // call that draws each face. That way, the OBJ's spec's "as many vertices as you want" implementation of faces
        // still works, but we can do it in one subroutine call. Probably wouldn't be as efficient as if the model was
        // all triangulated, but hey---OBJ is ancient. What more do you want from me, bro

        // ok time for bed, its nearly 5am again. yeehaw

        // Temp:
        // ------------------------------------

        // // Raw material data, indexed by name
        // let mut materials = HashMap::<Box<str>, ObjMaterial>::new();

        // // State for current parser; each new material forces a new element group, and each group also forces the same.
        // // Final names of these groups are not stored, but we do use them
        // let mut curr_material: Option<&str> = None;
        // let mut curr_group: Option<Box<str>> = None;

        // // Our final collection of material groups. Each different 'material' may use different maps and the like, and
        // // so will require a separate draw call. Each of those may refer to data in our main lists.
        // // let mut groups;

        // ------------------------------------

        // Raw vertex attribute data
        let mut v_data = Vec::new();
        let mut vt_data = Vec::new();
        let mut vn_data = Vec::new();

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
            // zero-length, and we did so after trimming, which guarantees that there is at least one thing pre-whitespace).
            let directive = line.split_whitespace().nth(0).unwrap();
            let rest = line[directive.len()..].trim_start(); // rest of the line

            match directive {
                "v" => v_data.push(parse_v(rest, &line_nums)?),
                "vt" => vt_data.push(parse_vt(rest, &line_nums)?),
                "vn" => vn_data.push(parse_vn(rest, &line_nums)?),
                "f" => {
                    // Parse all the vertex reference numbers on this line into actual indices into our three lists of
                    // vertex data
                    let indices = parse_f(rest, (v_data.len(), vt_data.len(), vn_data.len()), &line_nums);
                },
                "usemtl" => warn!("(unimplemented) 'usemtl' directive on {}", fmt_line_range(&line_nums)),
                "mtllib" => warn!("(unimplemented) 'mtllib' directive on {}", fmt_line_range(&line_nums)),
                _ => check_other(directive, &line_nums)?,
            }
        }

        // debug!(
        //     "finished parsing file {}. found {} vertices, {} normals, and {} tex-coords used by {} faces.",
        //     path.display(),
        //     data.vertices.len(),
        //     data.normals.len(),
        //     data.tex_coords.len(),
        //     data.faces.len(),
        // );

        // Ok(data)

        todo!();
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
fn parse_f<'a, 'b>(
    text: &'a str,
    current_counts: (usize, usize, usize),
    lines: &'a LineRange,
) -> impl Iterator<Item = Result<(usize, Option<usize>, Option<usize>), ObjParseError>> + 'a {
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
                // If it's positive, subtract 1 to get the 0-based index; ensure it doesn't overflow our list.
                Ok(i) if i > 0 => match usize::try_from(i).unwrap() - 1 {
                    i if i < vec_len => Ok(i),
                    _ => Err(ObjParseError::f_index_range(lines, vec_name, i, vec_len)),
                },
                // If it's negative, subtract it from the list size; ensure it doesn't wrap around below zero.
                Ok(i) if i < 0 => match vec_len.checked_sub((-i).try_into().unwrap()) {
                    Some(i) => Ok(i),
                    None => Err(ObjParseError::f_index_range(lines, vec_name, i, vec_len)),
                },
                // If it's neither, it's zero; out of range (OBJ vertex references are 1-based indices).
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

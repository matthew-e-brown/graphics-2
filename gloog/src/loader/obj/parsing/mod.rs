mod mtl;

use std::fs::File;
use std::io::BufReader;
use std::num::ParseFloatError;
use std::path::Path;
use std::sync::Arc;

use arrayvec::ArrayVec;
use gloog_math::{Vec2, Vec3};
use log::{debug, log, trace, warn};

use super::error::ObjParseError;
use super::{FaceElementData, ObjData};
use crate::loader::{fmt_line_range, lines_escaped, LineRange};


/// Current parser state as we run through the file.
///
/// OBJ parsing is state-based. Lines are "directives" which either introduce new vertices or modify current states.
/// States include things like which group things are a part of, current material information, and so on.
#[derive(Debug, Default)]
struct ParseState {
    cur_material: Option<Arc<str>>,
}


impl ObjData {
    pub fn load_from_file(path: impl AsRef<Path>) -> Result<ObjData, ObjParseError> {
        let mut data = ObjData::default(); // Final data that gets returned
        let state = ParseState::default(); // Other state that changes as we parse
        // NB: ^will need to be `mut` when material support is added

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
                "f" => read_f(rest, &mut data, &state, &line_nums)?,
                "mtllib" => warn!("(unimplemented) 'mtllib' directive on {}", fmt_line_range(&line_nums)),
                "usemtl" => warn!("(unimplemented) 'usemtl' directive on {}", fmt_line_range(&line_nums)),
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

        Ok(data)
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
fn read_v(text: &str, data: &mut Vec<Vec3>, lines: &LineRange) -> Result<(), ObjParseError> {
    // Take at most four; some files seem to specify extra `1` values after the w to force some viewers to get the
    // message (that's my guess anyways).
    let (floats, remaining) = read_ws_verts::<4>(text).or(ObjParseError::v_parse_err(lines, "v").into())?;

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
        let vec = [floats[0], floats[1], floats[2]].into();
        trace!("parsed vertex {vec:?}");
        data.push(vec);
        Ok(())
    }
}

/// Parses the body of a `vn` directive from an OBJ file.
fn read_vn(text: &str, data: &mut Vec<Vec3>, lines: &LineRange) -> Result<(), ObjParseError> {
    // Take as many as they provided, in a Vec. The spec says that there should always be three.
    let (floats, remaining) = read_ws_verts::<3>(text).or(ObjParseError::v_parse_err(lines, "vn").into())?;

    if floats.len() < 3 {
        Err(ObjParseError::v_too_small(lines, "vn", floats.len(), 3))
    } else if remaining > 0 {
        Err(ObjParseError::v_too_large(lines, "vn", floats.len() + remaining, 3))
    } else {
        // We know there's exactly three now
        let vec = [floats[0], floats[1], floats[2]].into();
        trace!("parsed normal {vec:?}");
        data.push(vec);
        Ok(())
    }
}

/// Parses the body of a `vn` directive from an OBJ file.
fn read_vt(text: &str, data: &mut Vec<Vec2>, lines: &LineRange) -> Result<(), ObjParseError> {
    let (floats, remaining) = read_ws_verts::<3>(text).or(ObjParseError::v_parse_err(lines, "vt").into())?;

    let vec = match floats.len() {
        0 => return Err(ObjParseError::v_too_small(lines, "vt", floats.len(), 1)),
        1 => [floats[0], 0.0].into(),
        2 => [floats[0], floats[1]].into(),
        3 if remaining == 0 => {
            debug!("found texture coord with 3rd dimension; 3rd dimension will be ignored");
            [floats[0], floats[1]].into()
        },
        n => return Err(ObjParseError::v_too_large(lines, "vt", n + remaining, 3)),
    };

    trace!("parsed tex-coord {vec:?}");

    data.push(vec);
    Ok(())
}

/// Parses the body of an `f` directive from an OBJ file.
fn read_f(
    text: &str,
    data: &mut ObjData,
    state: &ParseState,
    lines: &LineRange,
) -> Result<(), ObjParseError> {
    // Each face is a list of indices into vertex data. Indices may be of the form:
    // - `v`
    // - `v/vt`
    // - `v/vt/vn`
    // - `v//vn`
    // Each element in the list must be of the same form. Additionally, indices are 1-based and may be negative, which
    // represents an index from the current end of the list. There must be at least three vertices.

    // Iterator of each `(v/vt/vn) (v/vt/vn) (v/vt/vn) ...` in the current face:
    let mut vertex_references = text.split_whitespace().map(|v_refs_str| -> Result<_, _> {
        // Parses each of the slash-separated "vertex references" into a `usize`, taking into account any
        // negative/relative indexing. `vec_len` and `vec_name` are used for error-checking.
        let parse_v_ref = |ref_str: &str, vec_len: usize, vec_name: &'static str| -> Result<usize, _> {
            // Parse each component as a signed number, then convert that to a signed index.
            let ref_idx: isize = ref_str.parse().or(ObjParseError::f_parse_err(lines).into())?;
            let idx_err = ObjParseError::f_index_range(lines, vec_name, ref_idx, vec_len);
            if ref_idx > 0 {
                // If it's positive, subtract 1 to get the zero-based index; check if its within our range.
                match usize::try_from(ref_idx).unwrap() - 1 {
                    i if i < vec_len => Ok(i),
                    _ => Err(idx_err),
                }
            } else if ref_idx < 0 {
                // If it's negative, subtract it from the current length; check that it doesn't attempt to go before 0.
                let offset = usize::try_from(-ref_idx).unwrap();
                let idx = vec_len.checked_sub(offset).ok_or(idx_err)?;
                Ok(idx)
            } else {
                // A zero index is out of range.
                Err(idx_err)
            }
        };

        // Split on slashes to get each component:
        let mut indices = v_refs_str.split('/');

        // Then parse once for each of `v`, `vt`, and `vn`:
        let v_len = data.vertices.len();
        let vt_len = data.tex_coords.len();
        let vn_len = data.normals.len();

        // There must be at least one item (splitting "v" on '/' will just yield "v"), so we are safe to unwrap the
        // first number; error-check with `?` after doing so. `vt` and `vn` will be `None` if there weren't enough
        // slashes; `Option<&str> -> Option<Result<usize>>`, transpose to `Result<Option<usize>>` and `?`.
        let v = parse_v_ref(indices.next().unwrap(), v_len, "v")?;
        let vt = indices.next().map(|s| parse_v_ref(s, vt_len, "vt")).transpose()?;
        let vn = indices.next().map(|s| parse_v_ref(s, vn_len, "vn")).transpose()?;

        // There should only be 3 slash-separated components; if there are more remaining after parsing the first 3, we
        // have a problem.
        if indices.count() > 0 {
            Err(ObjParseError::f_parse_err(lines))
        } else {
            Ok((v, vt, vn))
        }
    });

    // Will compare all vertices to the first one, ensuring their slashes match
    let (v1, vt1, vn1) = vertex_references.next().ok_or(ObjParseError::f_too_few(lines, 0))??;

    // Seed our lists with the first vertex reference
    let mut v_list = vec![v1];
    let mut vt_list = vt1.map(|vt| vec![vt]);
    let mut vn_list = vn1.map(|vn| vec![vn]);

    // Then for each subsequent one:
    for parsed_v_refs in vertex_references {
        // Check if the tuple was parsed properly and push the first into its list.
        let (v, vt, vn) = parsed_v_refs?;
        v_list.push(v);

        // If both `vt` and the list of `vt` are Some, we're good; push into the list. Otherwise, the should both be
        // None.
        if let Some((v_ref, list)) = vt.zip(vt_list.as_mut()) {
            list.push(v_ref)
        } else if !vt.is_none() || !vt_list.is_none() {
            return Err(ObjParseError::f_mismatched(lines));
        }

        // Same goes for normals.
        if let Some((v_ref, list)) = vn.zip(vn_list.as_mut()) {
            list.push(v_ref)
        } else if !vn.is_none() || !vn_list.is_none() {
            return Err(ObjParseError::f_mismatched(lines));
        }
    }

    if v_list.len() < 3 {
        Err(ObjParseError::f_too_few(lines, v_list.len()))
    } else {
        data.faces.push(FaceElementData {
            material: state.cur_material.clone(),
            vertices: v_list,
            tex_coords: vt_list,
            normals: vn_list,
        });

        Ok(())
    }
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

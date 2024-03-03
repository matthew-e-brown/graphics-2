//! Module for loading models from OBJ files, optionally paired with MTL material files.
//!
//! Only all-polygonal models are supported are the moment (no free-form surfaces). These are those that use only `f`,
//! no `curv` or `surf` statements. Unsupported statements are simply ignored, though a warning is produced.
//!
//! For future reference, here are some of the documents I used for the OBJ and MTL specs:
//!
//! - <https://www.uhu.es/francisco.moreno/gii_rv/practicas/practica08/OBJ_SPEC.PDF>
//! - also: <https://paulbourke.net/dataformats/obj/> (missing math section)
//! - <https://paulbourke.net/dataformats/mtl/>

mod error;
mod mtl;

use std::collections::HashMap;
use std::fmt::Debug;
use std::fs::File;
use std::io::BufReader;
use std::num::{NonZeroUsize, ParseFloatError};
use std::ops::Range;
use std::path::Path;
use std::sync::{Arc, Weak};

use arrayvec::ArrayVec;
use bytemuck::{Pod, Zeroable};
use gloog_core::raw::types::GLuint;
use gloog_math::{Vec2, Vec3};
use image::{ImageBuffer, Luma, Rgba};
use log::{debug, info, log, trace};

use self::error::{ObjLoadError, ObjResult};
use crate::loader::obj::mtl::parse_mtl_file;
use crate::loader::{lines_escaped, LineRange};


// cspell:words curv interp stech ctech scrv cstype bmat newmtl usemtl mtllib maplib usemap


/// The constant that is used for primitive restarting in OpenGL.
const PRIMITIVE_RESTART: GLuint = GLuint::MAX;


/// A triple of indices into the three different sets of vertex data. Indices are 1-based to allow the optional values
/// to represent `None` using zero.
type FaceIndices = (NonZeroUsize, Option<NonZeroUsize>, Option<NonZeroUsize>);

/// A 32-bit RGBA image.
///
/// This type uses an [`Arc`] byte-slice as its backing store to allow for the data to be cached. Images are cached
/// using [`Weak`] references, though, and so they will not result in loaded textures staying in memory forever.
type RgbaImage = ImageBuffer<Rgba<u8>, Arc<[u8]>>;

/// An 8-bit grayscale image.
///
/// See [`RgbaImage`] for a note on the use of `Arc`.
type GrayImage = ImageBuffer<Luma<u8>, Arc<[u8]>>;

type CachedRgbaImage = ImageBuffer<Rgba<u8>, Weak<[u8]>>;
type CachedGrayImage = ImageBuffer<Luma<u8>, Weak<[u8]>>;

/// A wrapper for the various possible image files.
#[derive(Debug, Default)]
pub struct CachedImage {
    /// The value for the cached filename when it is being used for RGBA images, like ambient, diffuse, or specular
    /// maps.
    pub rgba: Option<CachedRgbaImage>,
    /// The value for the cached filename when it is being used for grayscale images, like specular exponents and alpha
    /// maps.
    pub gray: Option<CachedGrayImage>,
}


/// A drawable model comprised of data from an OBJ file.
///
/// The raw data behind this struct are behind [`Arc`] pointers, so this struct can be cloned cheaply without the need
/// for reallocating or re-parsing an entire model.
#[derive(Debug, Clone)]
pub struct ObjModel {
    /// Individual groups of faces which share a common material (and thus will share a single draw call). Each group
    /// contains a buffer of indices which index into [`Self::data`] using `glDrawElements`.
    groups: Arc<[ObjGroup]>,
    /// All raw vertex attribute data for this model. This data is referenced through indices in [`Self::indices`].
    data: Arc<[ObjVertex]>,
    /// All of the indices for this model. [`Self::groups`] contains where each `glDrawElements` call needs to start and
    /// stop.
    indices: Arc<[GLuint]>,
}

impl ObjModel {
    pub fn vertex_data(&self) -> &[ObjVertex] {
        &self.data[..]
    }

    pub fn index_data(&self) -> &[GLuint] {
        &self.indices[..]
    }

    pub fn groups(&self) -> &[ObjGroup] {
        &self.groups[..]
    }
}


/// Information about a group of faces to be drawn with a common material.
#[derive(Debug)]
pub struct ObjGroup {
    /// Contains information for how to set uniform information for this group's draw call.
    pub material: ObjMaterial,
    /// Where in [`ModelData::indices`] this particular group should be drawn.
    index_range: Range<usize>,
}

impl ObjGroup {
    pub fn indices(&self) -> Range<usize> {
        self.index_range.clone()
    }
}

/// A set of vertex attributes for a basic vertex.
#[repr(C)]
#[derive(Debug, Clone, Copy, Zeroable, Pod)]
pub struct ObjVertex {
    position: Vec3,
    normal: Vec3,
    tex_coord: Vec2,
}

macro_rules! vertex_offset {
    ($field:ident) => {{
        // We don't really need to care if the memory we're getting pointers to has been initialized properly, so we
        // just allocate some zeroes.
        let vert = unsafe { std::mem::MaybeUninit::<ObjVertex>::zeroed().assume_init() };
        let base = std::ptr::addr_of!(vert) as *const u8;
        let field = std::ptr::addr_of!(vert.$field) as *const u8;
        unsafe { field.offset_from(base) as usize }
    }};
}

impl ObjVertex {
    pub const STRIDE: isize = std::mem::size_of::<ObjVertex>() as isize;
    pub const OFFSET_POSITION: usize = vertex_offset!(position);
    pub const OFFSET_TEX_COORD: usize = vertex_offset!(tex_coord);
    pub const OFFSET_NORMAL: usize = vertex_offset!(normal);
}

/// Used to configure uniforms before executing draw call.
#[derive(Default, Clone)]
pub struct ObjMaterial {
    pub diffuse: Option<Vec3>,           // `Kd`
    pub ambient: Option<Vec3>,           // `Ka`
    pub specular: Option<Vec3>,          // `Ks`
    pub spec_pow: Option<f32>,           // `Ns`
    pub alpha: Option<f32>,              // `d` or `Tr`
    pub map_diffuse: Option<RgbaImage>,  // `map_Kd`
    pub map_ambient: Option<RgbaImage>,  // `map_Ka`
    pub map_specular: Option<RgbaImage>, // `map_Ks`
    pub map_spec_pow: Option<GrayImage>, // `map_Ns`
    pub map_alpha: Option<GrayImage>,    // `map_d`
    pub map_bump: Option<RgbaImage>,     // `bump` or `map_bump`
}

impl Debug for ObjMaterial {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut debug = f.debug_struct("ObjMaterial");
        let mut count = 0u32;

        /// Only bother printing if it's `Some`.
        macro_rules! field {
            ($field:ident) => {
                if let Some(value) = self.$field.as_ref() {
                    debug.field(stringify!($field), value);
                    count += 1;
                }
            };
        }

        field!(diffuse);
        field!(ambient);
        field!(specular);
        field!(spec_pow);
        field!(alpha);
        field!(map_diffuse);
        field!(map_ambient);
        field!(map_specular);
        field!(map_spec_pow);
        field!(map_alpha);
        field!(map_bump);

        if count == 0 {
            debug.field("all", &Option::<()>::None);
        }

        debug.finish()
    }
}


/// Grab everything in a line up to the first `#` (and also trim the starts and ends).
fn trim_comment(line: &str) -> &str {
    match line.find('#') {
        Some(i) => line[0..i].trim(),
        None => line[0..].trim(),
    }
}

impl ObjModel {
    pub fn from_file(
        path: impl AsRef<Path>,
        texture_cache: Option<&mut HashMap<Box<str>, CachedImage>>,
    ) -> ObjResult<Self> {
        let path = path.as_ref();
        let file = BufReader::new(File::open(path)?);

        // If we aren't given a texture cache, we need our own just for the local runtime of this function. We don't
        // need to bother initializing it if we were given one, so we delay the initialization until the check.
        let mut local_cache;
        let texture_cache = if let Some(passed) = texture_cache {
            passed
        } else {
            local_cache = HashMap::new();
            &mut local_cache
        };

        // Phase 1: Gathering data
        // --------------------------------------------------------------------

        let mut v_data = Vec::new(); // Vertex positions from `v` statements
        let mut vt_data = Vec::new(); // Texture coordinates from `vt` statements
        let mut vn_data = Vec::new(); // Vertex normals from `vn` statements

        let mut parsed_materials = Vec::new(); // List of all materials
        let mut material_indices = HashMap::new(); // Map of text name to index of material in list
        let mut curr_material = None; // Index of currently active material

        // List of faces and which materials they use
        let mut face_idx_buffer = Vec::new(); // List of all of the `FaceIndices` (each makes one final vertex)
        let mut face_vert_counts = Vec::new(); // Index `i` says how many verts face `i` uses
        let mut face_material_map = Vec::new(); // Index `i` names which material face `i` uses (as an index).

        for line_result in lines_escaped(file) {
            let (line_nums, line) = line_result?;
            let line = trim_comment(&line);

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

                    // Verify amount of vertices on this face
                    let pushed = face_idx_buffer.len() - before_push;
                    if pushed < 3 {
                        return Err(ObjLoadError::f_too_few(&line_nums, pushed));
                    } else if pushed > u32::MAX as usize {
                        return Err(ObjLoadError::f_too_many(&line_nums, pushed));
                    }

                    face_vert_counts.push(pushed as u32); // how many verts this face has; smaller number for mem usage
                    face_material_map.push(curr_material); // also track which material it used
                },
                "usemtl" => {
                    match material_indices.get(line) {
                        // `newmtl` and `usemtl` statements aren't supposed to support spaces, but who cares. The line
                        // is already trimmed and comments ignored, so we can just take the remainder of the line as the
                        // new name.
                        Some(idx) => curr_material = Some(*idx),
                        None => return Err(ObjLoadError::unknown_mtl(&line_nums, line)),
                    }
                },
                "mtllib" => {
                    // Don't forget that `mtllib` statements may have multiple filenames specified
                    for rel_path in line.split_whitespace() {
                        // Our base path is guaranteed to be a file (since we're in the middle of reading it), so
                        // `parent` is the directory.
                        let mtl_path = path.parent().unwrap().join(rel_path);
                        parse_mtl_file(mtl_path, &mut parsed_materials, &mut material_indices, texture_cache)?;
                    }
                },
                _ => check_other(directive, &line_nums)?,
            }
        }

        info!(
            "parsed v × {}, vt × {}, vn × {}, and f × {}",
            v_data.len(),
            vt_data.len(),
            vn_data.len(),
            face_vert_counts.len()
        );

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

        // List of final indices into final vertex data, grouped by (index of) material. Individual faces are separated
        // by `PRIMITIVE_RESTART_INDEX`.
        let mut index_groups = HashMap::new();
        let mut total_size = 0;

        for (vert_count, material_idx) in face_vert_counts.into_iter().zip(face_material_map.into_iter()) {
            // This loop runs once per face
            // ----------------------------------------------------------------

            let vert_count = vert_count as usize; // back to usize for indexing
            assert!(vert_count >= 3);

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
                let norm = ab.cross(&ac);

                // All the vertices in this face now have new surface normals; we have to push these into our data list
                // and update these vertices reference numbers.
                vn_data.push(norm);
                for (_, _, vn_idx) in &mut face_idx_buffer[..vert_count] {
                    *vn_idx = Some(NonZeroUsize::new(vn_data.len()).unwrap());
                }

                Some(norm)
            } else {
                None
            };

            // Grab the list that this face is going to push its indices into
            let index_list = index_groups.entry(material_idx).or_insert_with(|| Vec::new());

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
                        normal: vn,
                        tex_coord: vt,
                    });

                    // Already did a bounds-check earlier so we know this vertex's index won't overflow a `u32`
                    (vertex_data.len() - 1) as GLuint
                }));
            }

            // End of our face, so we want to restart our primitive rendering, too.
            index_list.push(PRIMITIVE_RESTART);
            total_size += vert_count + 1;
        }

        // Remove the last primitive reset index in each of the material groups, no need for it
        for index_list in index_groups.values_mut() {
            index_list.pop();
            total_size -= 1;
        }

        // Merge all the indices into one list, no longer grouped by material; keep the materials separate by
        // referencing indices into this new final buffer.
        let mut all_indices = Vec::with_capacity(total_size);

        let group_list = Vec::with_capacity(index_groups.len());
        let group_list = index_groups.into_iter().fold(group_list, |mut acc, (mtl_idx, indices)| {
            let material = mtl_idx.map(|i| parsed_materials[i].clone()).unwrap_or_default();
            let index_range = all_indices.len()..all_indices.len() + indices.len();

            all_indices.extend(indices);
            acc.push(ObjGroup { material, index_range });
            acc
        });

        info!(
            "finished loading model from `{}`: vertices: {}, indices: {}, material groups: {}",
            path.display(),
            vertex_data.len(),
            all_indices.len(),
            group_list.len(),
        );

        // Convert our indices into boxed slices, and finally grab the actual material values.
        Ok(ObjModel {
            data: vertex_data.into(),
            indices: all_indices.into(),
            groups: group_list.into(),
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


//`lines` is used for error-reporting
fn parse_v(text: &str, lines: &LineRange) -> ObjResult<Vec3> {
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
        // Some OBJ files seem to specify a bunch of extra '1's just to ensure the software sees it...? Maybe? Not sure.
        trace!("found 'v' directive with {} values; only the first 3 will be used", floats.len());

        let vertex = [floats[0], floats[1], floats[2]].into();
        Ok(vertex)
    } else {
        if floats.len() == 4 && floats[3] != 1.0 {
            debug!(
                "ignoring non-1.0 'w' component of '{}', since 'w' only affects free-form geometry (unsupported)",
                text
            );
        }

        // Take the first three and convert them into an array, then into a vector; we know there're at least 3.
        let vertex = [floats[0], floats[1], floats[2]].into();
        Ok(vertex)
    }
}

fn parse_vn(text: &str, lines: &LineRange) -> ObjResult<Vec3> {
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
        Ok(normal)
    }
}

fn parse_vt(text: &str, lines: &LineRange) -> ObjResult<Vec2> {
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

fn parse_f<'a>(
    text: &'a str,
    current_counts: (usize, usize, usize),
    lines: &'a LineRange,
) -> impl Iterator<Item = ObjResult<FaceIndices>> + 'a {
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
                    i if i <= vec_len => Ok(NonZeroUsize::new(i).unwrap()),
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
pub(crate) fn check_other(directive: &str, lines: &LineRange) -> ObjResult<()> {
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

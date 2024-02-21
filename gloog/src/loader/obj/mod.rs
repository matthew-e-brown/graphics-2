//! Module for loading models from OBJ files, optionally paired with MTL material files.
//!
//! Only all-polygonal models are supported are the moment (no free-form surfaces). These are those that use only `f`,
//! no `curv` or `surf` statements. Unsupported statements are simply ignored, though a warning is produced.

mod error;
mod parsing;

use std::collections::BTreeMap;
use std::sync::Arc;

use gloog_math::{Vec2, Vec3};

use crate::{Drawable, RawModelData, SceneObject};


// cspell:words curv interp stech ctech scrv cstype bmat usemtl mtllib maplib usemap

// Source for OBJ and MTL specs:
// - https://www.uhu.es/francisco.moreno/gii_rv/practicas/practica08/OBJ_SPEC.PDF
// - also: https://paulbourke.net/dataformats/obj/ (missing math section)
// - https://paulbourke.net/dataformats/mtl/


/// Raw data from an OBJ file, ready to be converted into an actual OpenGL-ready model.
#[derive(Debug, Default)]
pub struct ObjData {
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
    faces: Vec<FaceElementData>,
    /// List of all materials in the file, indexed by name.
    materials: BTreeMap<Arc<str>, MaterialData>,
}

/// A polygonal face (polygon) in an OBJ-file model.
#[derive(Debug)]
pub struct FaceElementData {
    /// The name of the material that this face uses.
    material: Option<Arc<str>>,
    /// A list of 0-based index into geometric vertex data.
    vertices: Vec<usize>,
    /// A list of 0-based indices into vertex texture coordinate data, if applicable.
    tex_coords: Option<Vec<usize>>,
    /// A list of 0-based indices into normal data, if applicable.
    normals: Option<Vec<usize>>,
}

#[derive(Debug, Default)]
pub struct MaterialData {
    pub ka: Option<Vec3>,
    pub kd: Option<Vec3>,
    pub ks: Option<Vec3>,
    pub ns: Option<f32>,
    pub opacity: Option<f32>,
    // todo: maps
}


impl RawModelData for ObjData {
    type Model = Model;

    fn decompose(self) -> Self::Model {
        // Construct a mapping of material names to lists of elements. Maybe erroneous for some objects, since it will
        // draw all like-material-ed faces together, but oh well.

        // Key'ed with an option to create one extra variant for `None` for when things don't have a group.
        let mut elements: BTreeMap<Option<Arc<str>>, Vec<Element>> = BTreeMap::new();

        // no wait ... we want one element for each material. So `BTreeMap<_, Element>`. Except, then we need Vec
        // instead of boxed slices. Might need a temp type or tuple here?
        //
        // idk. look at this again when it's not 5am. time for bed.

        for face in self.faces {
            let list = elements.entry(face.material).or_default();
            list.push(if true /* if untextured */ {
                todo!();
            } else {
                todo!();
            })
        }

        todo!();
    }
}


// consideration: Maybe we want to replace the combo of `ObjModel`, `GLTFModel` etc. plus a trait, with a more general
// `Model` *struct* that would reduce the need for generics. We'll see how things turn out when I implement loading of
// different error types.


/// A [drawable][Drawable] version of a model after being [loaded from an OBJ file][ObjData::load_from_file].
#[derive(Debug)]
pub struct Model {
    /// Base object for this model, which includes position and ID data.
    base: SceneObject,
    /// All elements of this object after being merged together based on their material.
    elems: Arc<[Element]>,
}


#[derive(Debug)]
pub enum Element {
    Untextured {
        /// List of all raw vertex data.
        attrs: Box<[UntexturedAttributes]>,
        /// Indices into vertex attributes for `glDrawElements`.
        elems: Box<[u32]>,
        /// Which material to use for this `glDrawElements` call.
        material: UntexturedMaterial,
    },
    Textured {
        attrs: Box<[TexturedAttributes]>,
        elems: Box<[u32]>,
        material: TexturedMaterial,
    },
}


#[derive(Debug)]
pub struct UntexturedMaterial {
    pub ambient: Vec3,
    pub diffuse: Vec3,
    pub specular: Vec3,
    pub spec_exp: f32,
    pub opacity: f32,
}


#[derive(Debug)]
pub struct TexturedMaterial {
    pub base: UntexturedMaterial,
    // todo: maps
}


#[repr(C)]
#[derive(Debug)]
pub struct UntexturedAttributes {
    pub position: Vec3,
    pub normal: Vec3,
}


#[repr(C)]
#[derive(Debug)]
pub struct TexturedAttributes {
    pub position: Vec3,
    pub tex_coord: Vec2,
    pub normal: Vec3,
}


impl AsRef<SceneObject> for Model {
    fn as_ref(&self) -> &SceneObject {
        &self.base
    }
}

impl Drawable for Model {}

pub mod loader;
pub mod model;

use std::borrow::Cow;
use std::fmt::Debug;
use std::sync::Mutex;

use gloog_math::{Vec3, Vec4};
pub use {gloog_core as core, gloog_math as math};


static NEXT_OBJECT_ID: Mutex<u32> = Mutex::new(0);


pub struct SceneObject {
    /// The ID of this scene object.
    ///
    /// Of the four components, opacity is incremented last in order to make it easier to debug the picking buffer. As
    /// in, the _highest_ byte of the ID becomes the 4th float, alpha. Additionally, we subtract our values from 255 so
    /// they start at 1.0 and transition downwards, giving us brighter colours.
    ///
    /// ```text
    /// let id: u32 = 0xAA_RR_GG_BB;
    /// let id = Vec4 {
    ///     w: (255 - 0xAA) / 255,
    ///     x: (255 - 0xRR) / 255,
    ///     y: (255 - 0xGG) / 255,
    ///     z: (255 - 0xBB) / 255,
    /// }
    /// ```
    id: u32,

    /// This object's ID formatted as a colour for picking operations.
    id_data: Vec4,

    /// A human-readable name for this object.
    name: Option<Cow<'static, str>>,

    /// This object's position in the scene in 3D.
    pos: Vec3,

    /// This object's X, Y, and Z scales in the scene.
    scl: Vec3,

    /// This object's _Euler angles_ for rotation in the scene.
    rot: Vec3,
}


impl Debug for SceneObject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.name.as_deref() {
            Some(name) => f.write_fmt(format_args!("SceneObject(\"{name}\")")),
            None => f.write_fmt(format_args!("SceneObject({:X})", self.id)),
        }
    }
}

impl Default for SceneObject {
    fn default() -> Self {
        let id = {
            let mut next_id = NEXT_OBJECT_ID.lock().expect("mutex poisoned");
            let out = *next_id;
            *next_id += 1;
            out
        };

        Self {
            id,
            id_data: Self::encode_id(id),
            name: None,
            pos: Vec3::default(),
            scl: Vec3::new(1.0, 1.0, 1.0),
            rot: Vec3::default(),
        }
    }
}

impl SceneObject {
    /// Encodes an object's ID into a colour.
    ///
    /// See [`decode_id`][Self::decode_id] for the inverse of this function.
    pub fn encode_id(id: u32) -> Vec4 {
        // NB: `>>` has higher precedence than `&`:
        // https://doc.rust-lang.org/reference/expressions.html#expression-precedence
        Vec4 {
            w: (255 - (id >> 24 & 0x000000ff)) as f32 / 255.0,
            x: (255 - (id >> 16 & 0x000000ff)) as f32 / 255.0,
            y: (255 - (id >> 08 & 0x000000ff)) as f32 / 255.0,
            z: (255 - (id >> 00 & 0x000000ff)) as f32 / 255.0,
        }
    }

    /// Decodes a colour into an object's ID.
    ///
    /// See [`encode_id`][Self::encode_id] for the inverse of this function.
    pub fn decode_id(id_data: Vec4) -> u32 {
        // 1. multiply each value to go from [0, 1] -> [0, 255]
        // 2. convert to u32, now between 0x00000000 and 0x000000ff
        // 3. subtract from 255 to return to original (eg., `18` would have gone to `237`; `255-237 = 18` again)
        // 4. shift back into place to construct u32
        let a = 255 - ((id_data.x * 255.0) as u32);
        let r = 255 - ((id_data.y * 255.0) as u32);
        let g = 255 - ((id_data.z * 255.0) as u32);
        let b = 255 - ((id_data.w * 255.0) as u32);
        (a << 24) | (r << 16) | (g << 8) | (b << 0)
    }

    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn id_data(&self) -> &Vec4 {
        &self.id_data
    }

    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    pub fn pos(&self) -> &Vec3 {
        &self.pos
    }

    pub fn scl(&self) -> &Vec3 {
        &self.scl
    }

    pub fn rot(&self) -> &Vec3 {
        &self.rot
    }
}


/// Not all [`SceneObjects`][SceneObject] are drawable.
pub trait Drawable: AsRef<SceneObject> {
    // ...
}


/// A model as it comes freshly parsed from a file.
pub trait RawModelData {
    /// The ready-to-render type that this data may turn into.
    type Model: Drawable;

    /// Converts this model into one that is ready to be fed to OpenGL.
    ///
    /// Called "decompose" because it may involve converting higher-level data structures into plain data, i.e.
    /// triangulating vertices. **This operation may be expensive!** Implementations should prioritize runtime
    /// performance and cache data/allocations aggressively.
    fn decompose(self) -> Self::Model;
}

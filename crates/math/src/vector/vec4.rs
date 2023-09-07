use bytemuck::{Pod, Zeroable};

use super::Vec3;


#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Pod, Zeroable)]
pub struct Vec4 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

super::impl_vector_basics!(Vec4, f32, 4, { 0: x, 1: y, 2: z, 3: w });

impl Vec4 {
    pub const UNIT_X: Vec4 = Vec4::new(1.0, 0.0, 0.0, 0.0);
    pub const UNIT_Y: Vec4 = Vec4::new(0.0, 1.0, 0.0, 0.0);
    pub const UNIT_Z: Vec4 = Vec4::new(0.0, 0.0, 1.0, 0.0);
    pub const UNIT_W: Vec4 = Vec4::new(0.0, 0.0, 0.0, 1.0);

    // --------------------------------------------------------------------------------------------

    /// Creates a new [`Vec4`] out of a [`Vec3`]'s `x`, `y`, and `z` components and a given `w` component.
    ///
    /// See also: [`Vec3::to4`]
    #[inline]
    pub const fn from3(xyz: Vec3, w: f32) -> Self {
        Self::new(xyz.x, xyz.y, xyz.z, w)
    }
}

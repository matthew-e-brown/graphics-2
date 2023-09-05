use bytemuck::{Pod, Zeroable};

use super::{Vec2, Vec3};


#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Pod, Zeroable)]
pub struct Vec4 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

super::impl_vector_basics!(Vec4, f32, 4; { 0: x, 1: y, 2: z, 3: w });

impl Vec4 {
    pub const UNIT_W: Vec4 = Vec4::new(0.0, 0.0, 0.0, 1.0);
    pub const UNIT_X: Vec4 = Vec4::new(1.0, 0.0, 0.0, 0.0);
    pub const UNIT_Y: Vec4 = Vec4::new(0.0, 1.0, 0.0, 0.0);
    pub const UNIT_Z: Vec4 = Vec4::new(0.0, 0.0, 1.0, 0.0);

    // --------------------------------------------------------------------------------------------

    /// Creates a new [`Vec4`] out of a [`Vec3`] and a float.
    #[inline]
    pub const fn from_xyz(xyz: Vec3, w: f32) -> Self {
        Self::new(xyz.x, xyz.y, xyz.z, w)
    }

    /// Creates a new [`Vec4`] out of a float and a [`Vec3`].
    #[inline]
    pub const fn from_yzw(x: f32, yzw: Vec3) -> Self {
        Self::new(x, yzw.x, yzw.y, yzw.z)
    }

    /// Creates a new [`Vec4`] out of a [`Vec2`] and two floats.
    #[inline]
    pub const fn from_xy(xy: Vec2, z: f32, w: f32) -> Self {
        Self::new(xy.x, xy.y, z, w)
    }

    /// Creates a new [`Vec4`] out of two floats and a [`Vec2`].
    #[inline]
    pub const fn from_zw(x: f32, y: f32, zw: Vec2) -> Self {
        Self::new(x, y, zw.x, zw.y)
    }

    /// Creates a new [`Vec4`] out of two [`Vec2`]s.
    #[inline]
    pub const fn from_xy_zw(xy: Vec2, zw: Vec2) -> Self {
        Self::new(xy.x, xy.y, zw.x, zw.y)
    }
}

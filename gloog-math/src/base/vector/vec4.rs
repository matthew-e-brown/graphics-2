use core::str::FromStr;

use bytemuck::{Pod, Zeroable};

use super::{parse_vec, ParseVecError, Vec3};


/// A four-dimensional vector of 32-bit floats.
///
/// This struct is `repr(C)`, so it is guaranteed to be identical to `[f32; 4]`.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Pod, Zeroable)]
pub struct Vec4 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

super::impl_vector_basics!(Vec4, f32, 4 (16), { 0: x, 1: y, 2: z, 3: w });

impl Vec4 {
    pub const UNIT_X: Vec4 = Vec4::new(1.0, 0.0, 0.0, 0.0);
    pub const UNIT_Y: Vec4 = Vec4::new(0.0, 1.0, 0.0, 0.0);
    pub const UNIT_Z: Vec4 = Vec4::new(0.0, 0.0, 1.0, 0.0);
    pub const UNIT_W: Vec4 = Vec4::new(0.0, 0.0, 0.0, 1.0);

    // --------------------------------------------------------------------------------------------

    /// Creates a new [`Vec4`] out of a [`Vec3`]'s `x`, `y`, and `z` components and a given `w` component.
    ///
    /// See also: [`Vec3::to_vec4`].
    #[inline]
    pub const fn from_vec3(xyz: Vec3, w: f32) -> Vec4 {
        Vec4::new(xyz.x, xyz.y, xyz.z, w)
    }

    /// Creates a new [`Vec3`] by ignoring this vector's `w` component.
    #[inline]
    pub const fn to_vec3(&self) -> Vec3 {
        Vec3::new(self.x, self.y, self.z)
    }
}

impl FromStr for Vec4 {
    type Err = ParseVecError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse_vec::<4>(s).map(|arr| arr.into())
    }
}

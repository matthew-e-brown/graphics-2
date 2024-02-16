use core::str::FromStr;

use bytemuck::{Pod, Zeroable};

use super::{parse_vec, ParseVecError, Vec3};


/// A two-dimensional vector of 32-bit floats.
///
/// This struct is `repr(C)`, so it is guaranteed to be identical to `[f32; 2]`.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Pod, Zeroable)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

super::impl_vector_basics!(Vec2, f32, 2 (8), { 0: x, 1: y });

impl Vec2 {
    pub const UNIT_X: Vec2 = Vec2::new(1.0, 0.0);
    pub const UNIT_Y: Vec2 = Vec2::new(0.0, 1.0);

    /// Creates a new [`Vector3D`] out of this vector's `x` and `y` components and a given `z` component.
    #[inline]
    pub const fn to3(&self, z: f32) -> Vec3 {
        Vec3::new(self.x, self.y, z)
    }
}

impl FromStr for Vec2 {
    type Err = ParseVecError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse_vec::<2>(s).map(|arr| arr.into())
    }
}

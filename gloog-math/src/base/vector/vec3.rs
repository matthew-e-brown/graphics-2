use core::str::FromStr;

use bytemuck::{Pod, Zeroable};

use super::{parse_vec, ParseVecError, Vec2, Vec4};


/// A three-dimensional vector of 32-bit floats.
///
/// This struct is `repr(C)`, so it is guaranteed to be identical to `[f32; 3]`.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Pod, Zeroable)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

super::impl_vector_basics!(Vec3, f32, 3 (12), { 0: x, 1: y, 2: z });

impl Vec3 {
    pub const UNIT_X: Vec3 = Vec3::new(1.0, 0.0, 0.0);
    pub const UNIT_Y: Vec3 = Vec3::new(0.0, 1.0, 0.0);
    pub const UNIT_Z: Vec3 = Vec3::new(0.0, 0.0, 1.0);

    /// Computes the cross product between this and another vector.
    #[inline]
    pub fn cross(&self, rhs: &Vec3) -> Vec3 {
        Vec3 {
            x: self.y * rhs.z - self.z * rhs.y,
            y: self.z * rhs.x - self.x * rhs.z,
            z: self.x * rhs.y - self.y * rhs.x,
        }
    }

    /// Computes the scalar triple product of vectors `a`, `b`, and `c`.
    ///
    /// The scalar triple product is equal to `a × b ⋅ c`, and is often written as `[a, b, c]`. It is also is the volume
    /// of the parallelepiped spanned by the three vectors.
    #[inline]
    pub fn scalar_triple(a: &Vec3, b: &Vec3, c: &Vec3) -> f32 {
        a.cross(b).dot(c)
    }

    // --------------------------------------------------------------------------------------------

    /// Creates a new [`Vec4`] from this vector's `x`, `y`, and `z` components and a given `w` component.
    ///
    /// See also: [`Vec4::from_vec3`].
    #[inline]
    pub const fn to_vec4(&self, w: f32) -> Vec4 {
        Vec4::new(self.x, self.y, self.z, w)
    }

    /// Creates a new [`Vec3`] out of a [`Vec4`] by ignoring its `w` component.
    ///
    /// See also: [`Vec4::to_vec3`].
    #[inline]
    pub const fn from_vec4(vec: Vec4) -> Vec3 {
        Vec3::new(vec.x, vec.y, vec.z)
    }

    /// Creates a new [`Vec3`] from a [`Vec2`] and a float.
    #[inline]
    pub const fn from_vec2(xy: Vec2, z: f32) -> Vec3 {
        Vec3::new(xy.x, xy.y, z)
    }
}

impl FromStr for Vec3 {
    type Err = ParseVecError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse_vec::<3>(s).map(|arr| arr.into())
    }
}

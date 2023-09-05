use bytemuck::{Pod, Zeroable};

use super::Vec2;


#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Pod, Zeroable)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

super::impl_vector_basics!(Vec3, f32, 3; { 0: x, 1: y, 2: z });

impl Vec3 {
    pub const UNIT_I: Vec3 = Vec3::new(1.0, 0.0, 0.0);
    pub const UNIT_J: Vec3 = Vec3::new(0.0, 1.0, 0.0);
    pub const UNIT_K: Vec3 = Vec3::new(0.0, 0.0, 1.0);

    /// Computes the cross product between this and another vector.
    ///
    /// This vector is the left-hand operand.
    #[inline]
    pub fn cross(&self, rhs: &Vec3) -> Vec3 {
        Vec3 {
            x: self.y * rhs.z - self.z * rhs.y,
            y: self.z * rhs.x - self.x * rhs.z,
            z: self.x * rhs.y - self.y * rhs.x,
        }
    }

    /// Computes the scalar triple product of `a`, `b`, and `c`.
    ///
    /// The scalar triple product of three vectors is the volume of the parallelepiped spanned by the vectors.
    ///
    /// This is equal to `a × b ⋅ c`. It is also written as `[a, b, c]`.
    #[inline]
    pub fn triple(a: &Vec3, b: &Vec3, c: &Vec3) -> f32 {
        a.cross(b).dot(c)
    }

    // --------------------------------------------------------------------------------------------

    /// Creates a new [`Vec3`] from a [`Vec2`] and a float.
    #[inline]
    pub const fn from_xy(xy: Vec2, z: f32) -> Self {
        Self::new(xy.x, xy.y, z)
    }

    /// Creates a new [`Vec3`] from a float and a [`Vec2`].
    #[inline]
    pub const fn from_yz(x: f32, yz: Vec2) -> Self {
        Self::new(x, yz.x, yz.y)
    }
}

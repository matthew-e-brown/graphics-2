use bytemuck::{Pod, Zeroable};

use super::{Vector2D, Vector4D};


/// A three-dimensional vector of 32-bit floats.
///
/// This struct is `repr(C)`, so it is guaranteed to be identical to `[f32; 3]`.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Pod, Zeroable)]
pub struct Vector3D {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

super::impl_vector_basics!(Vector3D, f32, 3 (12), { 0: x, 1: y, 2: z });

impl Vector3D {
    pub const UNIT_X: Vector3D = Vector3D::new(1.0, 0.0, 0.0);
    pub const UNIT_Y: Vector3D = Vector3D::new(0.0, 1.0, 0.0);
    pub const UNIT_Z: Vector3D = Vector3D::new(0.0, 0.0, 1.0);

    /// Computes the cross product between this and another vector.
    #[inline]
    pub fn cross(&self, rhs: &Vector3D) -> Vector3D {
        Vector3D {
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
    pub fn scalar_triple(a: &Vector3D, b: &Vector3D, c: &Vector3D) -> f32 {
        a.cross(b).dot(c)
    }

    // --------------------------------------------------------------------------------------------

    /// Creates a new [`Vector4D`] from this vector's `x`, `y`, and `z` components and a given `w` component.
    ///
    /// See also: [`Vector4D::from3`].
    #[inline]
    pub const fn to4(&self, w: f32) -> Vector4D {
        Vector4D::new(self.x, self.y, self.z, w)
    }

    /// Creates a new [`Vector3D`] from a [`Vector2D`] and a float.
    #[inline]
    pub const fn from2(xy: Vector2D, z: f32) -> Vector3D {
        Vector3D::new(xy.x, xy.y, z)
    }
}

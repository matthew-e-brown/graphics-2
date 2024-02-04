use bytemuck::{Pod, Zeroable};

use super::Vector3D;


/// A two-dimensional vector of 32-bit floats.
///
/// This struct is `repr(C)`, so it is guaranteed to be identical to `[f32; 2]`.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Pod, Zeroable)]
pub struct Vector2D {
    pub x: f32,
    pub y: f32,
}

super::impl_vector_basics!(Vector2D, f32, 2 (8), { 0: x, 1: y });

impl Vector2D {
    pub const UNIT_X: Vector2D = Vector2D::new(1.0, 0.0);
    pub const UNIT_Y: Vector2D = Vector2D::new(0.0, 1.0);

    /// Creates a new [`Vector3D`] out of this vector's `x` and `y` components and a given `z` component.
    #[inline]
    pub const fn to3(&self, z: f32) -> Vector3D {
        Vector3D::new(self.x, self.y, z)
    }
}

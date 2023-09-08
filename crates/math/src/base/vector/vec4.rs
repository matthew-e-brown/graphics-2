use bytemuck::{Pod, Zeroable};

use super::Vector3D;


/// A four-dimensional vector of 32-bit floats.
///
/// This struct is `repr(C)`, so it is guaranteed to be identical to `[f32; 4]`.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Pod, Zeroable)]
pub struct Vector4D {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

super::impl_vector_basics!(Vector4D, f32, 4 (16), { 0: x, 1: y, 2: z, 3: w });

impl Vector4D {
    pub const UNIT_X: Vector4D = Vector4D::new(1.0, 0.0, 0.0, 0.0);
    pub const UNIT_Y: Vector4D = Vector4D::new(0.0, 1.0, 0.0, 0.0);
    pub const UNIT_Z: Vector4D = Vector4D::new(0.0, 0.0, 1.0, 0.0);
    pub const UNIT_W: Vector4D = Vector4D::new(0.0, 0.0, 0.0, 1.0);

    // --------------------------------------------------------------------------------------------

    /// Creates a new [`Vector4D`] out of a [`Vector3D`]'s `x`, `y`, and `z` components and a given `w` component.
    ///
    /// See also: [`Vector3D::to4`].
    #[inline]
    pub const fn from3(xyz: Vector3D, w: f32) -> Vector4D {
        Vector4D::new(xyz.x, xyz.y, xyz.z, w)
    }
}

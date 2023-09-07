use bytemuck::{Pod, Zeroable};

use super::Vec3;


#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Pod, Zeroable)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

super::impl_vector_basics!(Vec2, f32, 2, { 0: x, 1: y });

impl Vec2 {
    pub const UNIT_I: Vec2 = Vec2::new(1.0, 0.0);
    pub const UNIT_J: Vec2 = Vec2::new(0.0, 1.0);

    /// Creates a new [`Vec3`] out of this vector's `x` and `y` components and a given `z` component.
    #[inline]
    pub const fn to3(&self, z: f32) -> Vec3 {
        Vec3::new(self.x, self.y, z)
    }
}

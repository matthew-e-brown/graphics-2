use bytemuck::{Pod, Zeroable};


#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Pod, Zeroable)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

super::impl_vector_basics!(Vec2, f32, 2; { 0: x, 1: y });

impl Vec2 {
    pub const UNIT_I: Vec2 = Vec2::new(1.0, 0.0);
    pub const UNIT_J: Vec2 = Vec2::new(0.0, 1.0);
}

//! Vectors.
//!
//! See [the crate-level documentation](crate) for general details that pertain to all data structures in the crate.

use bytemuck::{Pod, Zeroable};


#[cfg(test)] mod tests;


gloog_macro::create_vector! {
    #[derive(Copy, Debug, PartialEq)]
    pub struct Vec2;
    f32, 2;
}

gloog_macro::create_vector! {
    #[derive(Copy, Debug, PartialEq)]
    pub struct Vec3;
    f32, 3;
}

gloog_macro::create_vector! {
    #[derive(Copy, Debug, PartialEq)]
    pub struct Vec4;
    f32, 4;
}


gloog_macro::vector_impl_scalar_ops!(Vec2, f32, 2);
gloog_macro::vector_impl_scalar_ops!(Vec3, f32, 3);
gloog_macro::vector_impl_scalar_ops!(Vec4, f32, 4);

gloog_macro::vector_impl_self_ops!(Vec2, f32, 2);
gloog_macro::vector_impl_self_ops!(Vec3, f32, 3);
gloog_macro::vector_impl_self_ops!(Vec4, f32, 4);

gloog_macro::vector_impl_dot_product!(Vec2, f32, 2);
gloog_macro::vector_impl_dot_product!(Vec3, f32, 3);
gloog_macro::vector_impl_dot_product!(Vec4, f32, 4);


unsafe impl Zeroable for Vec2 {}
unsafe impl Zeroable for Vec3 {}
unsafe impl Zeroable for Vec4 {}

unsafe impl Pod for Vec2 {}
unsafe impl Pod for Vec3 {}
unsafe impl Pod for Vec4 {}

//! Matrices.
//!
//! See [the crate-level documentation](crate) for general details that pertain to all data structures in the crate.

use bytemuck::{Pod, Zeroable};


#[cfg(test)] mod tests;


gloog_macro::create_matrix! {
    #[derive(Copy, Debug, PartialEq, Pod, Zeroable)]
    pub struct Mat2;
    f32, 2, 2;
}

gloog_macro::create_matrix! {
    #[derive(Copy, Debug, PartialEq, Pod, Zeroable)]
    pub struct Mat3;
    f32, 3, 3;
}

gloog_macro::create_matrix! {
    #[derive(Copy, Debug, PartialEq, Pod, Zeroable)]
    pub struct Mat4;
    f32, 4, 4;
}

gloog_macro::matrix_impl_scalar_ops!(Mat2, f32, 2, 2);
gloog_macro::matrix_impl_scalar_ops!(Mat3, f32, 3, 3);
gloog_macro::matrix_impl_scalar_ops!(Mat4, f32, 4, 4);

gloog_macro::matrix_impl_self_ops!(Mat2, f32, 2, 2);
gloog_macro::matrix_impl_self_ops!(Mat3, f32, 3, 3);
gloog_macro::matrix_impl_self_ops!(Mat4, f32, 4, 4);

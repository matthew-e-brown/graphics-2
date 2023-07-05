//! Matrices.
//!
//! See [the crate-level documentation](crate) for general details that pertain to all data structures in the crate.

use bytemuck::{Pod, Zeroable};


#[cfg(test)] mod tests;


math_proc::create_matrix! {
    #[derive(Copy, Debug, PartialEq)]
    pub struct Mat2;
    f32, 2, 2;
}

math_proc::create_matrix! {
    #[derive(Copy, Debug, PartialEq)]
    pub struct Mat3;
    f32, 3, 3;
}

math_proc::create_matrix! {
    #[derive(Copy, Debug, PartialEq)]
    pub struct Mat4;
    f32, 4, 4;
}

math_proc::matrix_impl_scalar_ops!(Mat2, f32, 2, 2);
math_proc::matrix_impl_scalar_ops!(Mat3, f32, 3, 3);
math_proc::matrix_impl_scalar_ops!(Mat4, f32, 4, 4);

math_proc::matrix_impl_self_ops!(Mat2, f32, 2, 2);
math_proc::matrix_impl_self_ops!(Mat3, f32, 3, 3);
math_proc::matrix_impl_self_ops!(Mat4, f32, 4, 4);


unsafe impl Zeroable for Mat2 {}
unsafe impl Zeroable for Mat3 {}
unsafe impl Zeroable for Mat4 {}

unsafe impl Pod for Mat2 {}
unsafe impl Pod for Mat3 {}
unsafe impl Pod for Mat4 {}

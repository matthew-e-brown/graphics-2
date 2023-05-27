//! Mathematics data structures for working with OpenGL.
//!
//! Currently, this crate exports [*matrices*][mod@mat] and [*vectors*][mod@vec].
//!
//! # Operations
//!
//! All vectors and matrices have standard, component-wise operations defined on them for [`Mul`] and [`Div`] with their
//! inner type as the right-hand operand. They also have standard component-wise operations defined on them for [`Add`]
//! and [`Sub`] between themselves.
//!
//!
//! [`Add`]: std::ops::Add
//! [`Sub`]: std::ops::Sub
//! [`Mul`]: std::ops::Mul
//! [`Div`]: std::ops::Div
//!
//! # Conversions
//!
//! All vectors and matrices can also be converted to and from their inner representation using [`From`]. For example, a
//! `Vec3` can be created from three values using `Vec3::from([ a, b, c ])`.

/// Vectors.
///
/// See [the crate-level documentation](self) for general details that pertain to all data structures in the crate.
pub mod vec {
    math_proc::create_vector!(pub struct Vec2, f32, 2);
    math_proc::create_vector!(pub struct Vec3, f32, 3);
    math_proc::create_vector!(pub struct Vec4, f32, 4);

    math_proc::vector_impl_scalar_ops!(Vec2, f32, 2);
    math_proc::vector_impl_scalar_ops!(Vec3, f32, 3);
    math_proc::vector_impl_scalar_ops!(Vec4, f32, 4);

    math_proc::vector_impl_self_ops!(Vec2, f32, 2);
    math_proc::vector_impl_self_ops!(Vec3, f32, 3);
    math_proc::vector_impl_self_ops!(Vec4, f32, 4);
}

/// Matrices.
///
/// See [the crate-level documentation](self) for general details that pertain to all data structures in the crate.
pub mod mat {
    math_proc::create_matrix!(pub struct Mat2, f32, 2, 2);
    math_proc::create_matrix!(pub struct Mat3, f32, 3, 3);
    math_proc::create_matrix!(pub struct Mat4, f32, 4, 4);

    math_proc::matrix_impl_scalar_ops!(Mat2, f32, 2, 2);
    math_proc::matrix_impl_scalar_ops!(Mat3, f32, 3, 3);
    math_proc::matrix_impl_scalar_ops!(Mat4, f32, 4, 4);

    math_proc::matrix_impl_self_ops!(Mat2, f32, 2, 2);
    math_proc::matrix_impl_self_ops!(Mat3, f32, 3, 3);
    math_proc::matrix_impl_self_ops!(Mat4, f32, 4, 4);

    math_proc::matrix_impl_row_col_conversions!(Mat2, f32, 2, 2);
    math_proc::matrix_impl_row_col_conversions!(Mat3, f32, 3, 3);
    math_proc::matrix_impl_row_col_conversions!(Mat4, f32, 4, 4);
}

pub use mat::*;
pub use vec::*;

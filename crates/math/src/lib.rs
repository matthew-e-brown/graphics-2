//! Vectors.
//!
//! # Operators
//!
//! All vectors have standard, component-wise operations defined on them for [`Add`], [`Sub`], [`Mul`], and [`Div`] with
//! their inner type as the right-hand operand. For example, adding `10f32` to a [`Vec3`] will add 10 to each of its
//! components.
//!
//!
//! [`Add`]: std::ops::Add
//! [`Sub`]: std::ops::Sub
//! [`Mul`]: std::ops::Mul
//! [`Div`]: std::ops::Div

use math_proc::{create_matrix, create_vector};

create_vector!(pub struct Vec2, f32, 2);
create_vector!(pub struct Vec3, f32, 3);
create_vector!(pub struct Vec4, f32, 4);

create_matrix!(pub struct Mat2, f32, 2, 2);
create_matrix!(pub struct Mat3, f32, 3, 3);
create_matrix!(pub struct Mat4, f32, 4, 4);

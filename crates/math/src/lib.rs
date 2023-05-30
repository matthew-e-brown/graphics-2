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


pub mod vec;
pub mod mat;

pub use mat::*;
pub use vec::*;

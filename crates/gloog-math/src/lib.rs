//! Mathematics data structures for working with OpenGL.
//!
//! Currently, this crate contains [*matrices*][mod@mat] and [*vectors*][mod@vec]. See the documentations for those two
//! modules for more details. Their items are re-exported from this crate.
//!
//!
//! # Operators
//!
//! All vectors and matrices have standard, component-wise operations defined on them for [`Mul`] and [`Div`] with their
//! inner type as the right-hand operand (e.g., `Mat4 * f32`, all elements are multiplied by the same number). They also
//! have standard component-wise operations defined on them for [`Add`] and [`Sub`] between themselves (`Mat4 + Mat4`).
//!
//! More complex operations, like vector dot and cross products, are implemented as methods. They are called on the
//! left-hand operand (e.g., `vec3.dot(other_vec3)` or `Vec3::dot(vec3, other_vec3)`). These methods are detailed in the
//! [`mat`] and [`vec`] modules respectively.
//!
//!
//! # Conversions
//!
//! All vectors and matrices implement
//!
//! - [`From`] and [`Into`],
//! - [`AsRef`] and [`AsMut`], and
//! - [`Borrow`] and [`BorrowMut`]
//!
//! for their respective inner types, and the inner types implement them the other way. Notably, all structures also
//! implement bytemuck's [`Pod`] and [`Zeroable`]. These two trait implementations may be of more use for some cases.
//! All structures are `repr(transparent)`.
//!
//! With all of these trait implementations, casting and transforming back and forth between the underlying
//! representation (`Vec3` ⇄ `[f32; 3]` / `Mat2` ⇄ `[[f32; 2]; 2]`) should be simple.
//!
//!
//! [`Add`]:        core::ops::Add
//! [`Sub`]:        core::ops::Sub
//! [`Mul`]:        core::ops::Mul
//! [`Div`]:        core::ops::Div
//! [`From`]:       core::convert::From
//! [`Into`]:       core::convert::Into
//! [`AsRef`]:      core::convert::AsRef
//! [`AsMut`]:      core::convert::AsMut
//! [`Borrow`]:     core::borrow::Borrow
//! [`BorrowMut`]:  core::borrow::BorrowMut
//! [`Pod`]:        bytemuck::Pod
//! [`Zeroable`]:   bytemuck::Zeroable


// TODO: document matrix- and vector-specific functionality and operations in the docs of each of the two modules below.


/// Matrices.
///
/// See [the crate-level documentation](crate) for general details that pertain to all data structures in the crate.
pub mod mat;


/// Vectors.
///
/// See [the crate-level documentation](crate) for general details that pertain to all data structures in the crate.
pub mod vec;

pub use mat::*;
pub use vec::*;

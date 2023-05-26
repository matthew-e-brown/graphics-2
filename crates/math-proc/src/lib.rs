//! Macros for generating matrices and vectors.
//!
//! The inner workings of this crate are an implementation detail. The only reason there are doc-comments here at all is
//! because I'm insane and like to do things "properly" even if nobody else will ever read it.
//!
//! See the `math` crate for end-user documentation.

mod common;
mod matrix;
mod vector;

use syn::parse_macro_input;

use crate::matrix::{matrix_base, MatrixInput, MatrixRowColInput};
use crate::vector::{vector_base, VectorInput};

/// Creates a vector struct.
///
/// # Syntax
///
/// This macro takes three arguments:
///
/// - The struct's declaration "header" (`[visibility] struct <name>`);
/// - What type it should contain (any scalar type should work); and
/// - How many elements it should have (a [`usize`] literal).
///
/// ## Example
///
/// ```
/// create_vector!(pub(crate) struct IVec5, i32, 5);
/// ```
///
/// This will create:
///
/// ```
/// pub(crate) struct IVec5 {
///     v: [i32; 5],
/// }
/// ```
///
/// alongside some base functionality.
#[proc_macro]
pub fn create_vector(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as VectorInput);
    let output = vector_base(input);
    output.into()
}

/// Creates a matrix struct.
///
/// # Syntax
///
/// This macro takes four arguments:
///
/// - The struct's declaration (`[visibility] struct <name>`);
/// - What type it should contain, ([`f32`] and [`f64`] are the only two guaranteed to work);
/// - How many rows it should have (a [`usize`] literal); and
/// - How many columns it should have.
///
/// ## Example
///
/// ```
/// create_matrix!(pub struct DMat16x23, f64, 16, 23);
/// ```
///
/// This will create:
///
/// ```
/// pub struct DMat16x23 {
///     m: [[f64; 16]; 23],
/// }
/// ```
///
/// alongside some base functionality.
#[proc_macro]
pub fn create_matrix(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as MatrixInput);
    let output = matrix_base(input);
    output.into()
}


#[proc_macro]
pub fn impl_from_rows_and_cols(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as MatrixRowColInput);
    let mut output = matrix::impl_col_conversions(&input);
    output.extend(matrix::impl_row_conversions(&input));
    output.into()
}

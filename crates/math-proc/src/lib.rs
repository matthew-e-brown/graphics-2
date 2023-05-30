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
/// # use math_proc::create_vector;
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
    let input = parse_macro_input!(input as vector::CreationInput);
    let output = vector::create_base(input);
    output.into()
}


#[proc_macro]
pub fn vector_impl_scalar_ops(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as vector::SimpleInput);
    let output = vector::impl_scalar_ops(input);
    output.into()
}


#[proc_macro]
pub fn vector_impl_self_ops(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as vector::SimpleInput);
    let output = vector::impl_self_ops(input);
    output.into()
}


// ---------------------------------------------------------------------------------------------------------------------

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
/// # use math_proc::create_matrix;
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
    let input = parse_macro_input!(input as matrix::CreationInput);
    let output = matrix::create_base(input);
    output.into()
}


#[proc_macro]
pub fn matrix_impl_row_col_conversions(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as matrix::SimpleInput);
    let output = matrix::impl_row_col_conversions(input);
    output.into()
}


#[proc_macro]
pub fn matrix_impl_scalar_ops(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as matrix::SimpleInput);
    let output = matrix::impl_scalar_ops(input);
    output.into()
}


#[proc_macro]
pub fn matrix_impl_self_ops(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as matrix::SimpleInput);
    let output = matrix::impl_self_ops(input);
    output.into()
}

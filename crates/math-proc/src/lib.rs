use syn::parse::{Parse, ParseStream};
use syn::{parse_macro_input, Token};

mod matrix;
mod vector;


pub(crate) struct BaseInput {
    pub struct_vis: syn::Visibility,
    pub struct_name: syn::Ident,
    pub inner_type: syn::Type,
}

impl Parse for BaseInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // All visibility modifiers start with pub, and are then optionally followed by `(...)`. If there is no modifier
        // present, the struct has inherited visibility.
        let struct_vis = if input.peek(Token![pub]) {
            input.parse()?
        } else {
            syn::Visibility::Inherited
        };

        // Struct keyword, then struct name
        input.parse::<Token![struct]>()?;
        let struct_name = input.parse()?;

        // Comma, then inner type
        input.parse::<Token![,]>()?;
        let inner_type = input.parse()?;

        Ok(Self {
            struct_vis,
            struct_name,
            inner_type,
        })
    }
}

// ---------------------------------------------------------------------------------------------------------------------

pub(crate) struct VectorInput {
    pub base: BaseInput,
    pub num_elements: usize,
}

impl Parse for VectorInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // After base input, we want another comma
        let base = input.parse()?;
        input.parse::<Token![,]>()?;

        // Then just the number of elements
        let num_elements = input.parse::<syn::LitInt>()?.base10_parse()?;

        Ok(Self { base, num_elements })
    }
}

// ---------------------------------------------------------------------------------------------------------------------

pub(crate) struct MatrixInput {
    pub base: BaseInput,
    pub num_rows: usize,
    pub num_cols: usize,
}

impl Parse for MatrixInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let base = input.parse()?;
        input.parse::<Token![,]>()?;

        // Then we want two numbers this time, one for rows and one for columns
        let num_rows = input.parse::<syn::LitInt>()?.base10_parse()?;
        input.parse::<Token![,]>()?;
        let num_cols = input.parse::<syn::LitInt>()?.base10_parse()?;

        Ok(Self { base, num_rows, num_cols })
    }
}


// =====================================================================================================================
// =====================================================================================================================


/// Creates a vector struct.
///
/// # Syntax
///
/// This macro takes three arguments:
///
/// - The struct's declaration (`[visibility] struct <name>`);
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
    let output = vector::vector_base(input);
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
    let output = matrix::matrix_base(input);
    output.into()
}

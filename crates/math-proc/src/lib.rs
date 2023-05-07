use proc_macro2::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{parse_macro_input, Token};


/// Creates a matrix or matrix-like structure.
///
/// # Syntax
///
/// This macro takes three arguments:
///
/// * The name of the struct to create, optionally preceded by a visibility modifier;
/// * How many rows it should have (as a [`usize`]); and
/// * How many columns it should have (as a `usize`).
/// * What type it should contain (default [`f32`]).
#[proc_macro]
pub fn create_matrix(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as MatrixInput);
    let MatrixInput {
        struct_vis,
        struct_name,
        num_rows,
        num_cols,
        inner_type,
    } = &input;

    // Add to our list of implementations
    let mut impls = ImplStream::default();
    impl_index(&input, &mut impls);

    // Then flatten all of those down and create the final output
    let impls = impls.flatten(&input);
    proc_macro::TokenStream::from(quote! {
        #struct_vis struct #struct_name {
            // A "5x3 matrix" in math notation means one that is 5 tall and 3 wide; 5 rows, and 3 columns. If we want to
            // store in column-major order, that means we want 5 arrays, each 3 long. That gives us a list of columns in
            // order, each 3 long, of length 5. I always get it backwards in my head.
            data: [[#inner_type; #num_cols]; #num_rows],
        }

        #impls
    })
}


/// The input to the Matrix generator macro.
struct MatrixInput {
    num_rows: usize,
    num_cols: usize,
    struct_name: syn::Ident,
    struct_vis: syn::Visibility,
    inner_type: syn::Type,
}

impl Parse for MatrixInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let struct_vis = if input.peek(Token![pub]) {
            input.parse()?
        } else {
            syn::Visibility::Inherited
        };

        input.parse::<Token![struct]>()?;

        let struct_name = input.parse()?;

        input.parse::<Token![,]>()?;
        let num_rows = input.parse::<syn::LitInt>()?.base10_parse()?;

        input.parse::<Token![,]>()?;
        let num_cols = input.parse::<syn::LitInt>()?.base10_parse()?;

        let inner_type = if input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
            input.parse::<syn::Type>()?
        } else {
            // If they did not pass an inner type, use f32. Using `parse_str` is way easier than creating the struct
            // ourselves
            syn::parse_str("::core::primitive::f32").unwrap()
        };

        Ok(Self {
            struct_vis,
            struct_name,
            num_rows,
            num_cols,
            inner_type,
        })
    }
}


/// A wrapper for returning sets of implementations from helper functions. This is just so that, at the end, I can group
/// all of the functions for the "main" `impl` block into one `impl` block; that stops Rust Analyzer from reporting "600
/// implementations" if there are 600 functions.
#[derive(Default)]
struct ImplStream {
    /// This stream is expanded inside of an `impl #struct_name` at the end.
    pub main_funcs: TokenStream,
    /// This stream is expanded on its own.
    pub full_impls: TokenStream,
    /* cspell:words impls funcs */
}

impl ImplStream {
    /// Takes all the [`TokenStreams`][TokenStream] that this struct is holding onto and creates one final one.
    pub fn flatten(self, MatrixInput { struct_name, .. }: &MatrixInput) -> TokenStream {
        let Self { main_funcs, full_impls } = self;
        quote! {
            impl #struct_name { #main_funcs }
            #full_impls
        }
    }
}


/// Implements [`std::ops::Index`] and [`std::ops::IndexMut`] on the given matrix. Matrices are indexed Also creates a
/// `get` method for indexing with individual indices, if that syntax is desired.
fn impl_index(
    MatrixInput {
        struct_name,
        num_cols,
        inner_type,
        ..
    }: &MatrixInput,
    impls: &mut ImplStream,
) {
    let index_type: TokenStream;
    let indexer_expr: TokenStream;
    let get_function: TokenStream;

    if *num_cols == 1 {
        // If we have a column vector, we index with a single number
        index_type = quote! { ::core::primitive::usize };
        // We also need to then always get the zero'th column, since it's still a 2D array under the hood
        indexer_expr = quote! { [0][idx] };
        get_function = quote! {
            pub fn get(&self, idx: #index_type) -> &#inner_type {
                &self[idx]
            }
        };
    } else {
        // If we have a multi-dimensional matrix, we index with a tuple
        index_type = quote! { (::core::primitive::usize, ::core::primitive::usize) };
        // In math notation, indexing a matrix is done (row, column): A_21 is the second row down, and the first column
        // across. Because we're using column-major order, this indexing order is backwards; we have an array of
        // columns, meaning that the first column is array[0], and then we can retrieve the 2nd item in that column to
        // get row 2's value in that spot. Again, I always get this backwards, so I'm typing it out to confirm for
        // myself. The mathematical notation is also one-indexed, but we'll ignore that.
        indexer_expr = quote! { [idx.1][idx.0] };
        get_function = quote! {
            pub fn get(&self, idx_row: ::core::primitive::usize, idx_col: ::core::primitive::usize) -> &#inner_type {
                &self[(idx_row, idx_col)]
            }
        };
    }

    impls.main_funcs.extend(get_function);
    impls.full_impls.extend(quote! {
        impl ::std::ops::Index<#index_type> for #struct_name {
            type Output = #inner_type;
            fn index(&self, idx: #index_type) -> &Self::Output {
                &self.data #indexer_expr
            }
        }

        impl ::std::ops::IndexMut<#index_type> for #struct_name {
            fn index_mut(&mut self, idx: #index_type) -> &mut Self::Output {
                &mut self.data #indexer_expr
            }
        }
    });
}

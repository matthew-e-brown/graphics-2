//! This module has the implementation details for NxM matrices of floating-point values.

use indefinite::indefinite_article_only_capitalized;
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{parse_quote, Expr, ExprArray, Ident, LitInt, Token};

use crate::common::{self, BaseInput};

pub struct MatrixInput {
    pub base: BaseInput,
    pub num_rows: usize,
    pub num_cols: usize,
}

impl Parse for MatrixInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let base = input.parse()?;
        input.parse::<Token![,]>()?;

        // Then we want two numbers this time, one for rows and one for columns
        let num_rows = input.parse::<LitInt>()?.base10_parse()?;
        input.parse::<Token![,]>()?;
        let num_cols = input.parse::<LitInt>()?.base10_parse()?;

        Ok(Self { base, num_rows, num_cols })
    }
}

impl AsRef<BaseInput> for MatrixInput {
    fn as_ref(&self) -> &BaseInput {
        &self.base
    }
}

// ---------------------------------------------------------------------------------------------------------------------

pub fn matrix_base(input: MatrixInput) -> TokenStream {
    let MatrixInput {
        base: BaseInput {
            struct_vis,
            struct_name,
            inner_type,
        },
        num_rows,
        num_cols,
    } = &input;

    let doc = {
        let xy = format!("{num_rows}x{num_cols}");
        let an = indefinite_article_only_capitalized(&xy);
        let ty = quote!(#inner_type).to_string();
        format!("{an} {xy} matrix of `{ty}`s.")
    };

    let mut output = quote! {
        #[doc=#doc]
        #[doc=""]
        #[doc="See [the module-level documentation for more](self)."]
        #struct_vis struct #struct_name {
            // We want rows; cols. That gives us an array of columns, each containing one value for each row. This means
            // that to index the matrix using the mathematical convention of `M_rc` (row first), we need to index
            // backwards, `[col][row]`.
            m: [[#inner_type; #num_rows]; #num_cols],
        }
    };

    // TODO: Just like vectors, a bunch of these should probably be moved out of this `base` call and into their own
    // dedicated functions.

    output.extend(impl_constructor(&input));
    output.extend(impl_indexing(&input));
    output.extend(common::impl_scalar_ops(&input, num_rows * num_cols, |n| {
        let r = n % num_rows;
        let c = n / num_cols;
        parse_quote!(self[(#r, #c)])
    }));
    output.extend(common::impl_container_conversions(
        &input,
        &parse_quote! { [[#inner_type; #num_rows]; #num_cols] },
        &parse_quote!(m),
    ));

    output
}

fn impl_indexing(input: &MatrixInput) -> TokenStream {
    let MatrixInput {
        base: BaseInput { struct_name, inner_type, .. },
        ..
    } = input;

    quote! {
        impl std::ops::Index<(usize, usize)> for #struct_name {
            type Output = #inner_type;

            fn index(&self, idx: (usize, usize)) -> &Self::Output {
                &self.m[idx.1][idx.0]
            }
        }

        impl std::ops::IndexMut<(usize, usize)> for #struct_name {
            fn index_mut(&mut self, idx: (usize, usize)) -> &mut Self::Output {
                &mut self.m[idx.1][idx.0]
            }
        }
    }
}

fn impl_constructor(input: &MatrixInput) -> TokenStream {
    let MatrixInput {
        base: BaseInput { struct_name, inner_type, .. },
        num_rows,
        num_cols,
    } = input;

    let num_rows = *num_rows;
    let num_cols = *num_cols;
    let num_args = num_rows * num_cols;

    let param_types = std::iter::repeat(inner_type);
    let param_names: Vec<Ident> = (1..=num_args)
        .map(|n| Ident::new(&format!("m{n}"), Span::call_site()))
        .collect();

    // Create an array expression manually by looping through all the indices, column by column.
    let mut array_of_cols: ExprArray = parse_quote!([]);
    for c in 0..num_cols {
        // Create an array expression containing all the variable identities as expressions
        let mut row: ExprArray = parse_quote!([]);
        for r in 0..num_rows {
            let param = &param_names[c * num_cols + r];
            let param: Expr = parse_quote!(#param);
            row.elems.push(param);
        }

        array_of_cols.elems.push(Expr::Array(row));
    }

    quote! {
        impl #struct_name {
            #[doc="Creates a new matrix."]
            #[doc=""]
            #[doc="Parameters should be given in column-major order."]
            pub fn new(#(#param_names: #param_types),*) -> Self {
                Self {
                    m: #array_of_cols
                }
            }
        }
    }
}

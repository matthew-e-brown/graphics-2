//! This module has the implementation details for NxM matrices of floating-point values.

use indefinite::indefinite_article_only_capitalized;
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{parse_quote, Expr, ExprArray, Ident, LitInt, Token};

use crate::common::{self, BaseCreationInput, BaseSimpleInput};


// ---------------------------------------------------------------------------------------------------------------------
// Structs for macro input


fn parse_matrix_input<B: Parse>(input: ParseStream) -> syn::Result<(B, usize, usize)> {
    let base = input.parse()?;
    input.parse::<Token![,]>()?;
    let num_rows = input.parse::<LitInt>()?.base10_parse()?;
    input.parse::<Token![,]>()?;
    let num_cols = input.parse::<LitInt>()?.base10_parse()?;

    Ok((base, num_rows, num_cols))
}

/// Input for the base implementation of a matrix. Requires some extra things like visibility.
pub struct CreationInput {
    pub base: BaseCreationInput,
    pub num_rows: usize,
    pub num_cols: usize,
}

impl Parse for CreationInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let (base, num_rows, num_cols) = parse_matrix_input(input)?;
        Ok(Self { base, num_rows, num_cols })
    }
}

/// Input for all secondary matrix implementation functions; things that do not create the matrix.
pub struct SimpleInput {
    pub base: BaseSimpleInput,
    pub num_rows: usize,
    pub num_cols: usize,
}

impl Parse for SimpleInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let (base, num_rows, num_cols) = parse_matrix_input(input)?;
        Ok(Self { base, num_rows, num_cols })
    }
}


// ---------------------------------------------------------------------------------------------------------------------
// Helper functions

/// Converts a 1-dimensional index (i.e., constructor argument) into a 2-dimensional one.
#[inline]
fn index_1d_to_2d(idx: usize, num_rows: usize, num_cols: usize) -> (usize, usize) {
    let r = idx % num_rows;
    let c = idx / num_cols;
    (r, c)
}


// ---------------------------------------------------------------------------------------------------------------------
// Base implementation


pub fn create_base(input: CreationInput) -> TokenStream {
    let CreationInput {
        base: BaseCreationInput {
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

    // These features are always available on all matrices
    output.extend(impl_constructor(&input));
    output.extend(impl_indexing(&input));
    output.extend(common::impl_container_conversions(
        struct_name,
        &parse_quote!([[#inner_type; #num_rows]; #num_cols]),
        &parse_quote!(m),
    ));

    output
}

fn impl_indexing(input: &CreationInput) -> TokenStream {
    let CreationInput {
        base: BaseCreationInput { struct_name, inner_type, .. },
        ..
    } = input;

    quote! {
        impl ::core::ops::Index<(usize, usize)> for #struct_name {
            type Output = #inner_type;

            fn index(&self, idx: (usize, usize)) -> &Self::Output {
                &self.m[idx.1][idx.0]
            }
        }

        impl ::core::ops::IndexMut<(usize, usize)> for #struct_name {
            fn index_mut(&mut self, idx: (usize, usize)) -> &mut Self::Output {
                &mut self.m[idx.1][idx.0]
            }
        }
    }
}

fn impl_constructor(input: &CreationInput) -> TokenStream {
    let CreationInput {
        base: BaseCreationInput { struct_name, inner_type, .. },
        num_rows,
        num_cols,
    } = input;

    let num_rows = *num_rows;
    let num_cols = *num_cols;

    let param_types = std::iter::repeat(inner_type);
    let param_names: Vec<Vec<Ident>> = (0..num_rows)
        .map(|r| {
            (0..num_cols)
                .map(move |c| Ident::new(&format!("m{r}{c}"), Span::call_site()))
                .collect()
        })
        .collect();

    // Loop through the range of our (i, j) indices and build a 2D array of all of those parameter names
    let mut array_of_cols: ExprArray = parse_quote!([]);
    for c in 0..num_cols {
        // Since we're storing column-major, we want an array of columns; each column is made up of one value from each
        // row.
        let mut col: ExprArray = parse_quote!([]);
        for r in 0..num_rows {
            let param_name = &param_names[r][c];
            let param_expr: Expr = parse_quote!(#param_name);
            col.elems.push(param_expr);
        }

        array_of_cols.elems.push(Expr::Array(col));
    }

    // For a 3x3 matrix, `cols[0]` would look like the expression `[ m00, m10, m20 ]`.

    // Flatten our list of parameters from the 2D array we used to index correctly into one in order
    let param_names = param_names.iter().flat_map(|inner| inner.iter());

    // For a 3x3 matrix, the entire method body would just be `Self { m: [ [m0, m3, m6], [m1, m4, m7], [m2, m5, m8] ] }`
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

// ---------------------------------------------------------------------------------------------------------------------
// Additional implementations


pub fn impl_scalar_ops(input: SimpleInput) -> TokenStream {
    let SimpleInput {
        base: BaseSimpleInput { struct_name, inner_type },
        num_rows,
        num_cols,
    } = input;

    common::impl_cw_ops(
        common::CWOperatorSettings {
            lhs_type: &struct_name.into(),
            rhs_type: &inner_type.into(),
            lhs_indexer: Some(&|ident, n| {
                let (r, c) = index_1d_to_2d(n, num_rows, num_cols);
                parse_quote! { #ident[(#r, #c)] }
            }),
            rhs_indexer: None,
            constructor_arg_count: num_rows * num_cols,
        },
        &[common::BinaryOperator::Multiplication, common::BinaryOperator::Division],
    )
}


pub fn impl_self_ops(input: SimpleInput) -> TokenStream {
    let SimpleInput {
        base: BaseSimpleInput { struct_name, .. },
        num_rows,
        num_cols,
    } = input;

    let self_type = struct_name.into();
    common::impl_cw_ops(
        common::CWOperatorSettings {
            lhs_type: &self_type,
            rhs_type: &self_type,
            lhs_indexer: Some(&|ident, n| {
                let (r, c) = index_1d_to_2d(n, num_rows, num_cols);
                parse_quote! { #ident[(#r, #c)] }
            }),
            rhs_indexer: Some(&|ident, n| {
                let (r, c) = index_1d_to_2d(n, num_rows, num_cols);
                parse_quote! { #ident[(#r, #c)] }
            }),
            constructor_arg_count: num_rows * num_cols,
        },
        &[common::BinaryOperator::Addition, common::BinaryOperator::Subtraction],
    )
}


pub fn impl_row_col_conversions(input: SimpleInput) -> TokenStream {
    let SimpleInput {
        base: BaseSimpleInput { struct_name, inner_type },
        num_rows,
        num_cols,
    } = input;

    // -------------------------------------------------
    // Cols

    // c0 -> c{n}
    let param_names: Vec<Ident> = (0..num_cols).map(|c| Ident::new(&format!("c{c}"), Span::call_site())).collect();

    let cols = quote! {
        #[doc="Converts multiple columns into a single matrix."]
        #[doc=""]
        #[doc="Performing this conversion with vectors is essentially free. Since vectors are thin wrappers around"]
        #[doc="arrays and already represent column vectors, the vectors can be arranged in memory as-is, one after"]
        #[doc="the other."]
        pub fn from_cols<C>( #(#param_names: C),* ) -> Self
        where
            C: ::core::convert::Into<[#inner_type; #num_rows]>
        {
            Self::from([ #(#param_names.into()),* ])
        }
    };

    // -------------------------------------------------
    // Rows

    // This one is a bit less straightforward than `from_cols`. We can't just slot the vectors into place, since the
    // elements of a row do not lie next to one another in memory. Instead, we want a column made up of element 0 of
    // every row, then one made of element 1, and so on.

    // r0 -> r{n}
    let param_names: Vec<Ident> = (0..num_rows).map(|r| Ident::new(&format!("r{r}"), Span::call_site())).collect();
    // For each column, index each of our rows once
    let params_indexed = (0..num_cols).flat_map(|col| {
        param_names
            .iter()
            .map(move |row_ident| -> Expr { parse_quote!(#row_ident[#col]) })
    });

    let rows = quote! {
        #[doc="Converts multiple rows into a single matrix."]
        #[doc=""]
        #[doc="Because matrices are stored in column-major order, this operation cannot be done \"freely\" like"]
        #[doc="[`from_cols`][Self::from_cols] can. Each element must be copied into its place."]
        pub fn from_rows<R>( #(#param_names: R),* ) -> Self
        where
            R: ::core::convert::Into<[#inner_type; #num_cols]>
        {
            // We need to convert all of our parameter names first so that we can index them
            #( let #param_names = #param_names.into(); )*
            Self::new( #(#params_indexed),* )
        }
    };

    quote! {
        impl #struct_name {
            #rows
            #cols
        }
    }
}

//! This module has the implementation details for NxM matrices of floating-point values.

use indefinite::indefinite_article_only_capitalized;
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{parse_quote, Expr, ExprArray, Ident, LitInt, Path, Token, Type};

use crate::common::{self, BaseInput};

// ---------------------------------------------------------------------------------------------------------------------
// Main input struct

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
// Base implementation

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

fn impl_constructor(input: &MatrixInput) -> TokenStream {
    let MatrixInput {
        base: BaseInput { struct_name, inner_type, .. },
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
// Additional implementation, exposed to the caller as separate proc macros

pub struct MatrixRowColInput {
    /// The matrix we're implementing the `from_cols` or `from_rows` on.
    matrix: Path,
    /// The type that the vector and matrix both contain.
    inner_type: Type,
    num_rows: usize,
    num_cols: usize,
}

impl Parse for MatrixRowColInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // Matrix name,
        let matrix = input.parse()?;
        input.parse::<Token![,]>()?;

        // Vector name,
        let inner_type = input.parse()?;
        input.parse::<Token![,]>()?;

        let num_rows = input.parse::<LitInt>()?.base10_parse()?;
        input.parse::<Token![,]>()?;

        let num_cols = input.parse::<LitInt>()?.base10_parse()?;

        Ok(Self {
            matrix,
            inner_type,
            num_rows,
            num_cols,
        })
    }
}


pub fn impl_col_conversions(input: &MatrixRowColInput) -> TokenStream {
    let MatrixRowColInput {
        matrix,
        inner_type,
        num_rows,
        num_cols,
    } = input;

    let num_rows = *num_rows;
    let num_cols = *num_cols;

    // c0 -> c{n}
    let param_names: Vec<Ident> = (0..num_cols).map(|c| Ident::new(&format!("c{c}"), Span::call_site())).collect();

    quote! {
        impl #matrix {
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
        }
    }
}

pub fn impl_row_conversions(input: &MatrixRowColInput) -> TokenStream {
    let MatrixRowColInput {
        matrix,
        inner_type,
        num_rows,
        num_cols,
    } = input;

    let num_rows = *num_rows;
    let num_cols = *num_cols;

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

    quote! {
        impl #matrix {
            #[doc="Converts multiple rows into a single matrix."]
            #[doc=""]
            #[doc="Because matrices are stored in column-major order, this operation cannot be done \"freely\" like"]
            #[doc="[`from_cols`] can. Each element must be copied into its place."]
            pub fn from_rows<R>( #(#param_names: R),* ) -> Self
            where
                R: ::core::convert::Into<[#inner_type; #num_cols]>
            {
                // We need to convert all of our parameter names first so that we can index them
                #( let #param_names = #param_names.into(); )*
                Self::new( #(#params_indexed),* )
            }
        }
    }
}

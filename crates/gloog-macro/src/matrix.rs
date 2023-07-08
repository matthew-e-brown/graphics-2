//! This module has the implementation details for NxM matrices of floating-point values.

use indefinite::indefinite_article_only_capitalized;
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{parse_quote, Expr, ExprArray, Ident, LitInt, Token};

use crate::common::{self, BaseCreationInput, BaseSimpleInput};


// ---------------------------------------------------------------------------------------------------------------------
// Structs for macro input


/// Parses base macro input (be it [`BaseCreationInput`] or [`BaseSimpleInput`]), followed by matrix rows and columns.
fn parse_matrix_input<T: Parse>(input: ParseStream) -> syn::Result<(T, usize, usize)> {
    let base = input.parse()?;
    input.parse::<Token![,]>()?;
    let num_rows = input.parse::<LitInt>()?.base10_parse()?;
    input.parse::<Token![,]>()?;
    let num_cols = input.parse::<LitInt>()?.base10_parse()?;

    // Parse a semicolon optionally for the last parameter
    let _ = input.parse::<Token![;]>();

    Ok((base, num_rows, num_cols))
}


/// Input required to create an instance of a matrix.
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

/// Input required for most other matrix extension macros.
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
        base:
            BaseCreationInput {
                attributes,
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
        #(#attributes)*
        #[derive(::core::clone::Clone)]
        #[repr(transparent)]
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
    output.extend(impl_fallible_conversion(&input));
    output.extend(impl_row_col_conversions(&input));
    output.extend(common::impl_container_conversions(
        struct_name,
        &parse_quote!([[#inner_type; #num_rows]; #num_cols]),
        &parse_quote!(m),
    ));

    output
}


fn impl_constructor(input: &CreationInput) -> TokenStream {
    let CreationInput {
        base: BaseCreationInput { struct_name, inner_type, .. },
        num_rows,
        num_cols,
    } = input;

    let num_rows = *num_rows;
    let num_cols = *num_cols;

    // If either of our dimensions is greater than 9, separate the two with underscores (so that, for example, we get
    // 11_1 and 1_11 instead of 111 and 111, which would conflict).
    let sep = if num_rows > 9 || num_cols > 9 { "_" } else { "" };

    let param_types = std::iter::repeat(inner_type);
    let param_names: Vec<Vec<Ident>> = (1..=num_rows)
        .map(|r| {
            (1..=num_cols)
                .map(move |c| Ident::new(&format!("m_{r}{sep}{c}"), Span::call_site()))
                .collect()
        })
        .collect();

    // Loop through the range of our (i, j) indices and build a 2D array of all of those parameter names
    let mut array_of_cols: ExprArray = parse_quote!([]);

    for c in 0..num_cols {
        // Each `col` is made up of one element from each row
        let mut col: ExprArray = parse_quote!([]);
        for r in 0..num_rows {
            let param_name = &param_names[r][c];
            let param_expr: Expr = parse_quote!(#param_name);
            col.elems.push(param_expr);
        }

        array_of_cols.elems.push(Expr::Array(col));
    }

    // For a 3x3 matrix, `cols[0]` would look like the expression `[ m11, m21, m31 ]`.

    // Flatten our list of parameters from the 2D array we used to index correctly into one in order
    let param_names = param_names.iter().flat_map(|inner| inner.iter());

    // For a 3x3 matrix, the entire method body would just be `Self { m: [arrays] }`
    quote! {
        impl #struct_name {
            #[doc="Creates a new matrix."]
            #[doc=""]
            #[doc="Parameters should be given in **row-major order**. This is so that construction of matrices, when"]
            #[doc="laid out over multiple lines, lines up with their mathematical representation."]
            pub const fn new(#(#param_names: #param_types),*) -> Self {
                Self {
                    m: #array_of_cols
                }
            }
        }
    }
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


fn impl_fallible_conversion(input: &CreationInput) -> TokenStream {
    let CreationInput {
        base: BaseCreationInput { struct_name, inner_type, .. },
        num_rows,
        num_cols,
    } = input;

    // One `.try_into()?` for each column in our list of columns
    let col_attempts = (0..*num_cols).map(|n| quote! { m[#n].try_into()? });

    quote! {
        // Slice of arrays -> matrix
        impl<'a> ::core::convert::TryFrom<&'a [[#inner_type; #num_rows]]> for #struct_name {
            type Error = ::core::array::TryFromSliceError;

            fn try_from(value: &'a [[#inner_type; #num_rows]]) -> Result<Self, Self::Error> {
                let m = value.try_into()?;
                Ok(Self { m })
            }
        }

        // Slice of slices -> matrix
        impl<'a, 'b> TryFrom<&'a [&'b [#inner_type]]> for #struct_name {
            type Error = core::array::TryFromSliceError;

            fn try_from(value: &'a [&'b [#inner_type]]) -> Result<Self, Self::Error> {
                let m: [_; #num_cols] = value.try_into()?;
                let m = [ #(#col_attempts),* ];
                Ok(Self { m })
            }
        }
    }
}

fn impl_row_col_conversions(input: &CreationInput) -> TokenStream {
    let CreationInput {
        base: BaseCreationInput { struct_name, inner_type, .. },
        num_rows,
        num_cols,
        ..
    } = input;

    let num_cols = *num_cols;
    let num_rows = *num_rows;

    // -------------------------------------------------
    // Cols

    // c0 -> c{n}
    let param_names: Vec<Ident> = (0..num_cols).map(|c| Ident::new(&format!("c{c}"), Span::call_site())).collect();

    let from_cols = quote! {
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

        #[doc="Tries to convert multiple columns into a single matrix."]
        #[doc=""]
        #[doc="This method simply calls [`try_into`](TryInto::try_into) on all columns, then falls back on"]
        #[doc="[`from_cols`]."]
        pub fn try_from_cols<C>( #(#param_names: C),* ) -> Result<Self, C::Error>
        where
            C: ::core::convert::TryInto<[#inner_type; #num_rows]>
        {
            #( let #param_names = #param_names.try_into()?; )*
            Ok(Self::from_cols( #(#param_names),* ))
        }
    };

    // -------------------------------------------------
    // Rows

    // This one is a bit less straightforward than `from_cols`. We can't just slot the vectors into place, since the
    // elements of a row do not lie next to one another in memory. Instead, we want a column made up of element 0 of
    // every row, then one made of element 1, and so on.

    // r0 -> r{n}
    let param_names: Vec<Ident> = (0..num_rows).map(|r| Ident::new(&format!("r{r}"), Span::call_site())).collect();
    // For each row, get all the column values (since Mat::new takes row-major arguments)
    let params_indexed = param_names
        .iter()
        .flat_map(|row_ident| (0..num_cols).map(move |col| -> Expr { parse_quote!(#row_ident[#col]) }));

    let from_rows = quote! {
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

        #[doc="Tries to convert multiple rows into a single matrix."]
        #[doc=""]
        #[doc="This method simply calls [`try_into`][TryInto::try_into] on all rows, then falls back on [`from_rows`]."]
        pub fn try_from_rows<R>( #(#param_names: R),* ) -> Result<Self, R::Error>
        where
            R: ::core::convert::TryInto<[#inner_type; #num_cols]>
        {
            // We need to convert all of our parameter names first so that we can index them
            #( let #param_names = #param_names.try_into()?; )*
            Ok(Self::from_rows( #(#param_names),* ))
        }
    };

    quote! {
        impl #struct_name {
            #from_rows
            #from_cols
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
            total_elements: num_rows * num_cols,
        },
        &[common::BinaryOperator::Division],
        &[common::BinaryOperator::Multiplication], // implement multiplication for both mat*f32 and f32*mat
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
            total_elements: num_rows * num_cols,
        },
        &[common::BinaryOperator::Addition, common::BinaryOperator::Subtraction],
        &[], // self-ops are always "commutative" (mat + mat has the same impl as mat + mat)
    )
}

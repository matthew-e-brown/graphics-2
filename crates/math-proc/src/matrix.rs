//! This module has the implementation details for NxM matrices of floating-point values.

use indefinite::indefinite_article_only_capitalized;
use proc_macro2::TokenStream;
use quote::quote;

use crate::{BaseInput, MatrixInput};


pub(crate) fn matrix_base(input: MatrixInput) -> TokenStream {
    let MatrixInput {
        base: BaseInput {
            struct_vis,
            struct_name,
            inner_type,
        },
        num_rows,
        num_cols,
    } = input;

    let doc = {
        let dims = format!("{num_rows}x{num_cols}");
        let article = indefinite_article_only_capitalized(&dims);
        format!("{article} {dims} matrix of `{}`s.", quote!(#inner_type).to_string())
    };

    quote! {
        #[doc=#doc]
        #struct_vis struct #struct_name {
            // We want rows; cols. That gives us an array of columns, each containing one value for each row. This means
            // that to index the matrix using the mathematical convention of `M_rc` (row first), we need to index
            // backwards.
            m: [[#inner_type; #num_rows]; #num_cols],
        }
    }
}

//! This module has the implementation details for N-length vectors of arbitrary scalar values.

use indefinite::indefinite_article_only_capitalized;
use proc_macro2::TokenStream;
use quote::quote;

use crate::{BaseInput, VectorInput};


pub(crate) fn vector_base(input: VectorInput) -> TokenStream {
    let VectorInput {
        base: BaseInput {
            struct_vis,
            struct_name,
            inner_type,
        },
        num_elements,
    } = input;

    let doc = {
        let els = num_elements.to_string();
        let article = indefinite_article_only_capitalized(&els);
        format!(
            "{article} {els}-element column-vector of `{}`s.",
            quote!(#inner_type).to_string()
        )
    };

    quote! {
        #[doc=#doc]
        #struct_vis struct #struct_name {
            v: [#inner_type; #num_elements],
        }
    }
}

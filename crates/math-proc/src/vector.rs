//! This module has the implementation details for N-length vectors of arbitrary scalar values.

use indefinite::indefinite_article_only_capitalized;
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{parse_quote, Ident, LitInt, Token};

use crate::common::{self, BaseInput};

pub struct VectorInput {
    pub base: BaseInput,
    pub num_elements: usize,
}

impl Parse for VectorInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // After base input, we want another comma
        let base = input.parse()?;
        input.parse::<Token![,]>()?;

        // Then just the number of elements
        let num_elements = input.parse::<LitInt>()?.base10_parse()?;

        Ok(Self { base, num_elements })
    }
}

impl AsRef<BaseInput> for VectorInput {
    fn as_ref(&self) -> &BaseInput {
        &self.base
    }
}

// ---------------------------------------------------------------------------------------------------------------------

pub fn vector_base(input: VectorInput) -> TokenStream {
    let VectorInput {
        base: BaseInput {
            struct_vis,
            struct_name,
            inner_type,
        },
        num_elements,
    } = &input;

    let doc = {
        let el = num_elements.to_string();
        let an = indefinite_article_only_capitalized(&el);
        let ty = quote!(#inner_type).to_string();
        format!("{an} {el}-element column-vector of `{ty}`s.")
    };

    let mut output = quote! {
        #[doc=#doc]
        #[doc=""]
        #[doc="See [the module-level documentation for more](self)."]
        #struct_vis struct #struct_name {
            v: [#inner_type; #num_elements],
        }
    };

    // TODO: take a bunch of these out of here and make them separate macros, so that it's possible to create vector
    // structs with only some of these features. For example, a BVec would not support binary + operations with another
    // bool.

    output.extend(impl_constructor(&input));
    output.extend(impl_indexing(&input));
    output.extend(common::impl_scalar_ops(&input, input.num_elements, |n| parse_quote!(self[#n])));
    output.extend(common::impl_container_conversions(
        &input,
        &parse_quote! { [#inner_type; #num_elements] },
        &parse_quote!(v),
    ));

    output
}

fn impl_indexing(input: &VectorInput) -> TokenStream {
    let VectorInput {
        base: BaseInput { struct_name, inner_type, .. },
        ..
    } = input;

    quote! {
        impl ::core::ops::Index<usize> for #struct_name {
            type Output = #inner_type;

            fn index(&self, idx: usize) -> &Self::Output {
                &self.v[idx]
            }
        }

        impl ::core::ops::IndexMut<usize> for #struct_name {
            fn index_mut(&mut self, idx: usize) -> &mut Self::Output {
                &mut self.v[idx]
            }
        }
    }
}

fn impl_constructor(input: &VectorInput) -> TokenStream {
    let VectorInput {
        base: BaseInput { struct_name, inner_type, .. },
        num_elements,
    } = input;
    let num_args = *num_elements;

    let param_types = std::iter::repeat(inner_type);
    let param_names: Vec<Ident> = if num_args <= 4 {
        ["x", "y", "z", "w"]
            .into_iter()
            .take(num_args)
            .map(|s| Ident::new(s, Span::call_site()))
            .collect()
    } else {
        (1..=num_args)
            .map(|n| Ident::new(&format!("v{n}"), Span::call_site()))
            .collect()
    };

    quote! {
        impl #struct_name {
            #[doc="Creates a new vector."]
            pub fn new(#(#param_names: #param_types),*) -> Self {
                Self {
                    v: [ #(#param_names),* ],
                }
            }
        }
    }
}

//! This module has the implementation details for N-length vectors of arbitrary scalar values.

use indefinite::indefinite_article_only_capitalized;
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{parse_quote, ExprBinary, Ident, LitInt, Token};

use crate::common::{self, BaseCreationInput, BaseSimpleInput};


// ---------------------------------------------------------------------------------------------------------------------
// Structs for macro input


/// Parses base macro input (be it [`BaseCreationInput`] or [`BaseSimpleInput`]), followed by a number of elements.
fn parse_vector_input<T: Parse>(input: ParseStream) -> syn::Result<(T, usize)> {
    let base = input.parse()?;
    input.parse::<Token![,]>()?;
    let num_elements = input.parse::<LitInt>()?.base10_parse()?;

    // Parse a semicolon optionally for the last parameter
    let _ = input.parse::<Token![;]>();

    Ok((base, num_elements))
}

/// Input required to create an instance of a vector.
pub struct CreationInput {
    pub base: BaseCreationInput,
    pub num_elements: usize,
}

impl Parse for CreationInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let (base, num_elements) = parse_vector_input(input)?;
        Ok(Self { base, num_elements })
    }
}

/// Input required for most other vector extension macros.
pub struct SimpleInput {
    pub base: BaseSimpleInput,
    pub num_elements: usize,
}

impl Parse for SimpleInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let (base, num_elements) = parse_vector_input::<BaseSimpleInput>(input)?;
        Ok(Self { base, num_elements })
    }
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
        #(#attributes)*
        #struct_vis struct #struct_name {
            v: [#inner_type; #num_elements],
        }
    };

    // These features are always available on all vectors
    output.extend(impl_constructor(&input));
    output.extend(impl_indexing(&input));
    output.extend(common::impl_container_conversions(
        struct_name,
        &parse_quote!([#inner_type; #num_elements]),
        &parse_quote!(v),
    ));

    output
}

fn impl_indexing(input: &CreationInput) -> TokenStream {
    let CreationInput {
        base: BaseCreationInput { struct_name, inner_type, .. },
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

fn impl_constructor(input: &CreationInput) -> TokenStream {
    let CreationInput {
        base: BaseCreationInput { struct_name, inner_type, .. },
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


// ---------------------------------------------------------------------------------------------------------------------
// Additional implementations


pub fn impl_scalar_ops(input: SimpleInput) -> TokenStream {
    let SimpleInput {
        base: BaseSimpleInput { struct_name, inner_type },
        num_elements,
    } = input;

    common::impl_cw_ops(
        common::CWOperatorSettings {
            lhs_type: &struct_name.into(),
            rhs_type: &inner_type.into(),
            lhs_indexer: Some(&|ident, n| parse_quote! { #ident[#n] }),
            rhs_indexer: None,
            total_elements: num_elements,
        },
        &[common::BinaryOperator::Division],
        &[common::BinaryOperator::Multiplication], // vec*f32 and f32*vec
    )
}


pub fn impl_self_ops(input: SimpleInput) -> TokenStream {
    let SimpleInput {
        base: BaseSimpleInput { struct_name, .. },
        num_elements,
    } = input;

    let self_type = struct_name.into();
    common::impl_cw_ops(
        common::CWOperatorSettings {
            lhs_type: &self_type,
            rhs_type: &self_type,
            lhs_indexer: Some(&|ident, n| parse_quote! { #ident[#n] }),
            rhs_indexer: Some(&|ident, n| parse_quote! { #ident[#n] }),
            total_elements: num_elements,
        },
        &[common::BinaryOperator::Addition, common::BinaryOperator::Subtraction],
        &[], // self-ops are always commutative for free (vec + vec is the same as vec + vec)
    )
}


pub fn impl_dot_product(input: SimpleInput) -> TokenStream {
    let SimpleInput {
        base: BaseSimpleInput { struct_name, inner_type },
        num_elements,
    } = input;

    let terms = (0..num_elements).map(|i| -> ExprBinary {
        parse_quote! { self[#i] * rhs[#i] }
    });

    quote! {
        impl #struct_name {
            pub fn dot(&self, rhs: &Self) -> #inner_type {
                #( #terms )+*
            }
        }
    }
}

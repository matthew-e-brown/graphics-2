//! This module has the implementation details for N-length vectors of arbitrary scalar values.

use indefinite::indefinite_article_only_capitalized;
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::Ident;

use crate::{BaseInput, VectorInput};


pub(crate) fn vector_base(input: VectorInput) -> TokenStream {
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
    output.extend(impl_array_conversions(&input));
    output.extend(impl_scalar_ops(&input));
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
    let num_elements = *num_elements;

    let param_types = std::iter::repeat(inner_type);
    let param_names: Vec<Ident> = if num_elements <= 4 {
        ["x", "y", "z", "w"]
            .into_iter()
            .take(num_elements)
            .map(|s| Ident::new(s, Span::call_site()))
            .collect()
    } else {
        (1..=num_elements)
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


fn impl_array_conversions(input: &VectorInput) -> TokenStream {
    let VectorInput {
        base: BaseInput { struct_name, inner_type, .. },
        num_elements,
    } = input;
    let num_elements = *num_elements;
    let inner_type = quote!{ [#inner_type; #num_elements] };

    quote! {
        // &Vec3 as &[f32; 3]
        impl ::core::convert::AsRef<#inner_type> for #struct_name {
            fn as_ref(&self) -> &#inner_type {
                &self.v
            }
        }

        // &mut Vec3 as &mut [f32; 3]
        impl ::core::convert::AsMut<#inner_type> for #struct_name {
            fn as_mut(&mut self) -> &mut #inner_type {
                &mut self.v
            }
        }

        // [f32; 3] -> Vec3
        impl ::core::convert::From<#inner_type> for #struct_name {
            fn from(value: #inner_type) -> Self {
                #struct_name { v: value }
            }
        }

        // Vec3 -> [f32; 3]
        impl ::core::convert::From<#struct_name> for #inner_type {
            fn from(value: #struct_name) -> Self {
                value.v
            }
        }

        // &Vec3 -> &[f32; 3]
        impl<'a> ::core::convert::From<&'a #struct_name> for &'a #inner_type {
            fn from(value: &'a #struct_name) -> Self {
                &value.v
            }
        }

        // &mut Vec3 -> &mut [f32; 3]
        impl<'a> ::core::convert::From<&'a mut #struct_name> for &'a mut #inner_type {
            fn from(value: &'a mut #struct_name) -> Self {
                &mut value.v
            }
        }
    }
}


fn impl_scalar_ops(input: &VectorInput) -> TokenStream {
    use quote::quote as q;

    let VectorInput {
        base: BaseInput { struct_name, inner_type, .. },
        num_elements,
    } = input;
    let num_elements = *num_elements;

    #[rustfmt::skip]
    let operators = [
        (q!{ Add }, q!{ add }, q!{ + }),
        (q!{ Sub }, q!{ sub }, q!{ - }),
        (q!{ Mul }, q!{ mul }, q!{ * }),
        (q!{ Div }, q!{ div }, q!{ / }),
    ];

    /*
     * For each operator, we need one `impl` for of these configurations:
     * 1. Vec + f32        2. &Vec + f32      3. Vec + &f32      4. &Vec + &f32
     */
    #[rustfmt::skip]
    let operator_configs = [
        /* bounds      , left-hand param       , right-hand param      */
        (q!{ <      > }, q!{     #struct_name }, q!{     #inner_type  }),
        (q!{ <'a    > }, q!{ &'a #struct_name }, q!{     #inner_type  }),
        (q!{ <    'b> }, q!{     #struct_name }, q!{ &'b #inner_type  }),
        (q!{ <'a, 'b> }, q!{ &'a #struct_name }, q!{ &'b #inner_type  }),
    ];

    let mut output = TokenStream::new();

    for (operator_trait, operator_func, operator_token) in &operators {
        for (trait_bounds, lhs_type, rhs_type) in &operator_configs {
            // All of these operators involve constructing a new vector with updated values inside; we can just call
            // `Self::new` with varying arguments.
            let new_args = (0..num_elements).map(|n| q! { self[#n] #operator_token rhs });

            let op_impl = quote! {
                impl #trait_bounds ::core::ops::#operator_trait<#rhs_type> for #lhs_type {
                    type Output = #struct_name;

                    fn #operator_func(self, rhs: #rhs_type) -> Self::Output {
                        <#struct_name>::new( #(#new_args),* )
                    }
                }
            };

            output.extend(op_impl);
        }
    }

    output
}

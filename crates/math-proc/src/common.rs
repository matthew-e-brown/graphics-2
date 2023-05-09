use proc_macro2::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{BinOp, ExprIndex, Generics, Ident, Member, Path, Token, Type, Visibility};


pub struct BaseInput {
    pub struct_vis: Visibility,
    pub struct_name: Ident,
    pub inner_type: Type,
}

impl Parse for BaseInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // All visibility modifiers start with pub, and are then optionally followed by `(...)`. If there is no modifier
        // present, the struct has inherited visibility.
        let struct_vis = if input.peek(Token![pub]) { input.parse()? } else { Visibility::Inherited };

        // Struct keyword, then struct name
        input.parse::<Token![struct]>()?;
        let struct_name = input.parse()?;

        // Comma, then inner type
        input.parse::<Token![,]>()?;
        let inner_type = input.parse()?;

        Ok(Self {
            struct_vis,
            struct_name,
            inner_type,
        })
    }
}

// ---------------------------------------------------------------------------------------------------------------------

/// Implements [`Add`][std::ops::Add], [`Sub`][std::ops::Sub], [`Mul`][std::ops::Mul], and [`Div`][std::ops::Div] as
/// component-wise operations for the given input parameters.
///
/// - `num_args` is the number of arguments this object's constructor takes.
/// - `indexer` is a function that should return an expression providing an index into the container at a given point.
pub fn impl_scalar_ops<I, F>(input: I, num_args: usize, mut indexer: F) -> TokenStream
where
    I: AsRef<BaseInput>,
    F: FnMut(usize) -> ExprIndex,
{
    use syn::parse_quote as pq;
    let BaseInput { struct_name, inner_type, .. } = input.as_ref();

    #[rustfmt::skip]
    let operators: [(Path, Ident, BinOp); 4] = [
        (pq!{ ::core::ops::Add }, pq!{ add }, pq!{ + }),
        (pq!{ ::core::ops::Sub }, pq!{ sub }, pq!{ - }),
        (pq!{ ::core::ops::Mul }, pq!{ mul }, pq!{ * }),
        (pq!{ ::core::ops::Div }, pq!{ div }, pq!{ / }),
    ];

    /*
     * For each operator, we need one `impl` for of these configurations:
     * 1. Vec + f32        2. &Vec + f32      3. Vec + &f32      4. &Vec + &f32
     *
     * Which means we need several different configurations of lifetimes and reference vs. non-reference types.
     */
    #[rustfmt::skip]
    let operator_configs: [(Generics, Type, Type); 4] = [
        (pq!{ <      > }, pq!{     #struct_name }, pq!{     #inner_type }),
        (pq!{ <'a    > }, pq!{ &'a #struct_name }, pq!{     #inner_type }),
        (pq!{ <    'b> }, pq!{     #struct_name }, pq!{ &'b #inner_type }),
        (pq!{ <'a, 'b> }, pq!{ &'a #struct_name }, pq!{ &'b #inner_type }),
    ];

    let mut output = TokenStream::new();
    for (trait_path, operator_func, operator_token) in &operators {
        for (trait_bounds, lhs_type, rhs_type) in &operator_configs {
            // All of these operators involve constructing a new vector with updated values inside; we can just call
            // `Self::new` with varying arguments.
            let new_args = (0..num_args).map(|n| {
                let indexed = indexer(n);
                quote! { #indexed #operator_token rhs }
            });

            let op_impl = quote! {
                impl #trait_bounds #trait_path<#rhs_type> for #lhs_type {
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

/// Creates all sorts of conversions that allow the vector or matrix to be converted back and forth from the type it
/// contains.
///
/// - `container_type` should be the type of the entire type that the vector or matrix contains, like `[f32; 2]`.
/// - `container_name` is the name of that container as a member of the vector or matrix.
pub fn impl_container_conversions<I>(input: I, container_type: &Type, container_name: &Member) -> TokenStream
where
    I: AsRef<BaseInput>,
{
    let BaseInput { struct_name, .. } = input.as_ref();

    quote! {
        // &Vec3 as &[f32; 3], or &Mat4 as &[[f32; 4]; 4]
        impl ::core::convert::AsRef<#container_type> for #struct_name {
            fn as_ref(&self) -> &#container_type {
                &self.#container_name
            }
        }

        // &mut Vec3 as &mut [f32; 3] or &mut Mat4 as &mut [[f32; 4]; 4]
        impl ::core::convert::AsMut<#container_type> for #struct_name {
            fn as_mut(&mut self) -> &mut #container_type {
                &mut self.#container_name
            }
        }

        // [f32; 3] -> Vec3, or [[f32; 4]; 4] -> Mat4
        impl ::core::convert::From<#container_type> for #struct_name {
            fn from(value: #container_type) -> Self {
                #struct_name { #container_name: value }
            }
        }

        // Vec3 -> [f32; 3], or Mat4 -> [[f32; 4]; 4]
        impl ::core::convert::From<#struct_name> for #container_type {
            fn from(value: #struct_name) -> Self {
                value.#container_name
            }
        }

        // &Vec3 -> &[f32; 3], or &Mat4 -> &[[f32; 4]; 4]
        impl<'a> ::core::convert::From<&'a #struct_name> for &'a #container_type {
            fn from(value: &'a #struct_name) -> Self {
                &value.#container_name
            }
        }

        // &mut Vec3 -> &mut [f32; 3], or &mut Mat4 -> &mut [[f32; 4]; 4]
        impl<'a> ::core::convert::From<&'a mut #struct_name> for &'a mut #container_type {
            fn from(value: &'a mut #struct_name) -> Self {
                &mut value.#container_name
            }
        }
    }
}

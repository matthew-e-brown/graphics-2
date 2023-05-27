use proc_macro2::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{parse_quote, BinOp, Expr, Generics, Ident, Member, Path, Token, Type, TypePath, Visibility};


pub struct BaseCreationInput {
    pub struct_vis: Visibility,
    pub struct_name: Ident,
    pub inner_type: Type,
}

impl Parse for BaseCreationInput {
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

pub struct BaseSimpleInput {
    pub struct_name: TypePath,
    pub inner_type: Type,
}

impl Parse for BaseSimpleInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let struct_name = input.parse()?;
        input.parse::<Token![,]>()?;
        let inner_type = input.parse()?;

        Ok(Self { struct_name, inner_type })
    }
}


// ---------------------------------------------------------------------------------------------------------------------

/// One component-wise operator to be implemented.
#[derive(Copy, Clone)]
pub enum BinaryOperator {
    Addition,
    Subtraction,
    Multiplication,
    Division,
}

impl BinaryOperator {
    /// Gets the pieces needed to write out this operator's trait implementation.
    ///
    /// - The trait name (`::core::ops::Add`)
    /// - The trait's function (`add`)
    /// - The actual binary operator to apply to the LHS and RHS (`+`)
    #[rustfmt::skip]
    pub fn to_pieces(&self) -> (Path, Ident, BinOp) {
        use syn::parse_quote as pq;
        match self {
            Self::Addition          => (pq! { ::core::ops::Add }, pq! { add }, pq! { + }),
            Self::Subtraction       => (pq! { ::core::ops::Sub }, pq! { sub }, pq! { - }),
            Self::Multiplication    => (pq! { ::core::ops::Mul }, pq! { mul }, pq! { * }),
            Self::Division          => (pq! { ::core::ops::Div }, pq! { div }, pq! { / }),
        }
    }
}


/// Settings for implementation of component-wise operations.
///
/// Using a reference type for the LHS- or RHS-type parameters will result in operators being implemented for `&T` and
/// `&&T` instead of `T` and `&T`.
pub struct CWOperatorSettings<'a> {
    pub lhs_type: &'a Type,
    pub rhs_type: &'a Type,
    pub lhs_indexer: Option<&'a dyn Fn(&Ident, usize) -> Expr>,
    pub rhs_indexer: Option<&'a dyn Fn(&Ident, usize) -> Expr>,
    pub constructor_arg_count: usize,
}


/// Uses [`CWOperatorSettings`] to implement component-wise (CW) operations between one structure and another. Can be
/// used either for Vec->f32 or Vec->Vec and similar. Does **not** implement the opposing (f32->Vec) operator in
/// non-symmetric cases.
pub fn impl_cw_ops(settings: CWOperatorSettings, operators: &[BinaryOperator]) -> TokenStream {
    use syn::parse_quote as pq;

    let CWOperatorSettings {
        lhs_type: lhs_base_type,
        rhs_type: rhs_base_type,
        lhs_indexer,
        rhs_indexer,
        constructor_arg_count,
    } = settings;

    /*
     * For each operator, we need one `impl` for of these configurations (using Vec+f32 as an example):
     * 1. Vec + f32        2. &Vec + f32      3. Vec + &f32      4. &Vec + &f32
     *
     * Which means we need several different configurations of lifetimes and reference vs. non-reference types.
     */
    #[rustfmt::skip]
    let impl_configs: [(Generics, Type, Type); 4] = [
        (pq!{ <      > }, pq!{     #lhs_base_type }, pq!{     #rhs_base_type }),
        (pq!{ <'a    > }, pq!{ &'a #lhs_base_type }, pq!{     #rhs_base_type }),
        (pq!{ <    'b> }, pq!{     #lhs_base_type }, pq!{ &'b #rhs_base_type }),
        (pq!{ <'a, 'b> }, pq!{ &'a #lhs_base_type }, pq!{ &'b #rhs_base_type }),
    ];

    let operators = operators.iter().map(|o| o.to_pieces());
    let mut output = TokenStream::new();

    let lhs_ident = parse_quote! { lhs };
    let rhs_ident = parse_quote! { rhs };

    for (op_trait, op_func, op_op) in operators {
        for (op_bounds, lhs_type, rhs_type) in &impl_configs {
            let constructor_args = (0..constructor_arg_count).map(|n| {
                let lhs_expr = lhs_indexer
                    .as_ref()
                    .map(|f| f(&lhs_ident, n))
                    .unwrap_or_else(|| parse_quote! { #lhs_ident });
                let rhs_expr = rhs_indexer
                    .as_ref()
                    .map(|f| f(&rhs_ident, n))
                    .unwrap_or_else(|| parse_quote! { #rhs_ident });
                quote! { #lhs_expr #op_op #rhs_expr }
            });

            output.extend(quote! {
                impl #op_bounds #op_trait<#rhs_type> for #lhs_type {
                    type Output = #lhs_base_type;

                    fn #op_func(self, rhs: #rhs_type) -> Self::Output {
                        let lhs = self;
                        <#lhs_base_type>::new( #(#constructor_args),* )
                    }
                }
            });
        }
    }

    output
}


/// Creates all sorts of conversions that allow the vector or matrix to be converted back and forth from the type it
/// contains.
///
/// - `wrapped_type` is the entire type that the vector or matrix contains (`[f32; 2]` instead of `f32`).
/// - `member_name` is the name of that container as a member of the vector or matrix (`.m` or `.data`, for example).
pub fn impl_container_conversions(struct_name: &Ident, wrapped_type: &Type, member_name: &Member) -> TokenStream {
    quote! {
        // - `&Vec3` as `&[f32; 3]`
        // - `&Mat4` as `&[[f32; 4]; 4]`
        impl ::core::convert::AsRef<#wrapped_type> for #struct_name {
            fn as_ref(&self) -> &#wrapped_type {
                &self.#member_name
            }
        }

        // - `&mut Vec3` as `&mut [f32; 3]`
        // - `&mut Mat4` as `&mut [[f32; 4]; 4]`
        impl ::core::convert::AsMut<#wrapped_type> for #struct_name {
            fn as_mut(&mut self) -> &mut #wrapped_type {
                &mut self.#member_name
            }
        }

        // - `[f32; 3]` -> `Vec3`
        // - `[[f32; 4]; 4]` -> `Mat4`
        impl ::core::convert::From<#wrapped_type> for #struct_name {
            fn from(value: #wrapped_type) -> Self {
                #struct_name { #member_name: value }
            }
        }

        // - `Vec3` -> `[f32; 3]`
        // - `Mat4` -> `[[f32; 4]; 4]`
        impl ::core::convert::From<#struct_name> for #wrapped_type {
            fn from(value: #struct_name) -> Self {
                value.#member_name
            }
        }

        // - `&Vec3` -> `&[f32; 3]`
        // - `&Mat4` -> `&[[f32; 4]; 4]`
        impl<'a> ::core::convert::From<&'a #struct_name> for &'a #wrapped_type {
            fn from(value: &'a #struct_name) -> Self {
                &value.#member_name
            }
        }

        // - `&mut Vec3` -> `&mut [f32; 3]`
        // - `&mut Mat4` -> `&mut [[f32; 4]; 4]`
        impl<'a> ::core::convert::From<&'a mut #struct_name> for &'a mut #wrapped_type {
            fn from(value: &'a mut #struct_name) -> Self {
                &mut value.#member_name
            }
        }
    }
}

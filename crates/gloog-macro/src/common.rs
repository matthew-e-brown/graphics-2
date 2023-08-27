use proc_macro2::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{parse_quote, Attribute, BinOp, Expr, Generics, Ident, Member, Path, Token, Type, TypePath, Visibility};


/// The inputs required for the creation of any struct through a macro. Extended by both [`matrix`] and [`vector`]
/// modules. At the very least, we need to know how public to make the struct, what it should be called, and what type
/// it should hold.
pub struct BaseCreationInput {
    pub attributes: Vec<Attribute>,
    pub struct_vis: Visibility,
    pub struct_name: Ident,
    pub inner_type: Type,
}

impl Parse for BaseCreationInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // Parse as many attributes as we can
        let attributes = input.call(Attribute::parse_outer).unwrap_or_default();

        // Parse the `pub(...)` if there; otherwise, just inherit visibility
        let struct_vis = input.parse().unwrap_or(Visibility::Inherited);

        // Struct keyword, then struct name
        input.parse::<Token![struct]>()?;
        let struct_name = input.parse()?;

        // Semicolon separates "parameters" from the base
        input.parse::<Token![;]>()?;

        // Then inner type
        let inner_type = input.parse()?;

        Ok(Self {
            attributes,
            struct_vis,
            struct_name,
            inner_type,
        })
    }
}


/// The inputs required for extending any macro that was created by [`BaseCreationInput`]. At the very least, we always
/// need to know the name of the struct to extend and the type that it contains.
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
    pub fn pieces(&self) -> (Path, Ident, BinOp) {
        use syn::parse_quote as pq;
        match self {
            Self::Addition          => (pq! { ::core::ops::Add }, pq! { add }, pq! { + }),
            Self::Subtraction       => (pq! { ::core::ops::Sub }, pq! { sub }, pq! { - }),
            Self::Multiplication    => (pq! { ::core::ops::Mul }, pq! { mul }, pq! { * }),
            Self::Division          => (pq! { ::core::ops::Div }, pq! { div }, pq! { / }),
        }
    }

    /// The same as [`to_pieces`][Self::to_pieces], but for assignment operators (`+=`).
    #[rustfmt::skip]
    pub fn assignment_pieces(&self) -> (Path, Ident, BinOp) {
        use syn::parse_quote as pq;
        match self {
            Self::Addition          => (pq! { ::core::ops::AddAssign }, pq! { add_assign }, pq! { += }),
            Self::Subtraction       => (pq! { ::core::ops::SubAssign }, pq! { sub_assign }, pq! { -= }),
            Self::Multiplication    => (pq! { ::core::ops::MulAssign }, pq! { mul_assign }, pq! { *= }),
            Self::Division          => (pq! { ::core::ops::DivAssign }, pq! { div_assign }, pq! { /= }),
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
    pub total_elements: usize,
    // /// Whether or not to use the RHS type as the output for the given operator traits. Setting this to true also
    // /// disables creating assignment traits (e.g., [`MulAssign`][core::ops::MulAssign]), because the RHS type cannot be
    // /// assigned directly to the LHS, which is how assignments are done.
    // pub use_rhs_as_output: bool,
}


/// Uses [`CWOperatorSettings`] to implement component-wise (CW) operations between one structure and another. Can be
/// used either for Vec->f32 or Vec->Vec and similar.
pub fn impl_cw_ops(
    settings: CWOperatorSettings,
    operators_noncommutative: &[BinaryOperator],
    operators_commutative: &[BinaryOperator],
) -> TokenStream {
    use syn::parse_quote as pq;

    let CWOperatorSettings {
        lhs_type: lhs_base_type,
        rhs_type: rhs_base_type,
        lhs_indexer,
        rhs_indexer,
        total_elements,
    } = settings;

    let operators_commutative = operators_commutative.iter().map(|op| (true, op));
    let operators_noncommutative = operators_noncommutative.iter().map(|op| (false, op));

    let operators = operators_commutative.chain(operators_noncommutative);

    /*
     * For each operator, we need one `impl` for of these configurations (using Vec+f32 as an example):
     * 1. Vec + f32         2. &Vec + f32           3. Vec + &f32           4. &Vec + &f32
     *
     * Which means we need several different configurations of lifetimes and reference vs. non-reference types. We don't
     * need to define anything for `mut` references because they automatically coerce into immutable references.
     */
    #[rustfmt::skip]
    let ref_configs: [(Generics, Type, Type); 4] = [
        (pq! { <      > }, pq! {     #lhs_base_type }, pq! {     #rhs_base_type }),
        (pq! { <'l    > }, pq! { &'l #lhs_base_type }, pq! {     #rhs_base_type }),
        (pq! { <    'r> }, pq! {     #lhs_base_type }, pq! { &'r #rhs_base_type }),
        (pq! { <'l, 'r> }, pq! { &'l #lhs_base_type }, pq! { &'r #rhs_base_type }),
    ];

    /*
     * For each assignment operator, we need one `impl` for each of the following (again with the same example):
     * 1. Vec += f32        2. &mut Vec += f32      3. Vec += &f32      4. &mut Vec += &f32
     *
     * This is the exact same as the basic operators, but we need mutable references to the LHS.
     */
    #[rustfmt::skip]
    let mut_configs: [(Generics, Type, Type); 4] = [
        (pq! { <      > }, pq! {         #lhs_base_type }, pq! {     #rhs_base_type }),
        (pq! { <'l    > }, pq! { &'l mut #lhs_base_type }, pq! {     #rhs_base_type }),
        (pq! { <    'r> }, pq! {         #lhs_base_type }, pq! { &'r #rhs_base_type }),
        (pq! { <'l, 'r> }, pq! { &'l mut #lhs_base_type }, pq! { &'r #rhs_base_type }),
    ];

    let mut output = TokenStream::new();

    let lhs_ident = parse_quote! { lhs };
    let rhs_ident = parse_quote! { rhs };

    // Takes the given binary operator and generates a list of `lhs[idx] [op] rhs[idx]` for all components of the
    // structures.
    let generate_expressions = |op_op: BinOp| {
        // Capture identities by reference here, instead of in the inner `move` closure
        let lhs_ident = &lhs_ident;
        let rhs_ident = &rhs_ident;
        (0..total_elements).map(move |n| {
            let lhs_expr = lhs_indexer.as_ref().map(|f| f(lhs_ident, n)).unwrap_or_else(|| pq!(#lhs_ident));
            let rhs_expr = rhs_indexer.as_ref().map(|f| f(rhs_ident, n)).unwrap_or_else(|| pq!(#rhs_ident));
            quote! { #lhs_expr #op_op #rhs_expr }
        })
    };

    for (is_commutative, operator) in operators {
        // General operators
        for (op_bounds, lhs_type, rhs_type) in &ref_configs {
            let (op_trait, op_func, op_op) = operator.pieces();
            let constructor_args = generate_expressions(op_op).collect::<Vec<_>>();

            // Build a bunch of constructor arguments that all look like `lhs[idx] [op] rhs[idx]`. Then we'll pass all
            // of those to `LHS::new(...)` at once. They will come out as if written inline.
            output.extend(quote! {
                impl #op_bounds #op_trait<#rhs_type> for #lhs_type {
                    type Output = #lhs_base_type;

                    fn #op_func(self, #rhs_ident: #rhs_type) -> Self::Output {
                        let #lhs_ident = self;
                        <#lhs_base_type>::new( #(#constructor_args),* )
                    }
                }
            });

            if is_commutative {
                output.extend(quote! {
                    impl #op_bounds #op_trait<#lhs_type> for #rhs_type {
                        type Output = #lhs_base_type;

                        fn #op_func(self, #lhs_ident: #lhs_type) -> Self::Output {
                            let #rhs_ident = self;
                            <#lhs_base_type>::new( #(#constructor_args),* )
                        }
                    }
                });
            }
        }

        // Assignment variants
        for (op_bounds, lhs_type, rhs_type) in &mut_configs {
            let (op_trait, op_func, op_op) = operator.assignment_pieces();
            let assignment_statements = generate_expressions(op_op);

            output.extend(quote! {
                impl #op_bounds #op_trait<#rhs_type> for #lhs_type {
                    fn #op_func(&mut self, #rhs_ident: #rhs_type) {
                        let lhs = self;
                        #(#assignment_statements;)*
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
/// - `wrapped_type` is the entire type that the vector or matrix wraps (e.g., `[f32; 2]` instead of `f32`).
/// - `member_name` is the name of that container as a member of the vector or matrix (`.m` or `.data`, for example).
pub fn impl_container_conversions(struct_name: &Ident, wrapped_type: &Type, member_name: &Member) -> TokenStream {
    quote! {
        // -------------------------------------------------------------------------------------
        // Reference conversions (free)
        // -------------------------------------------------------------------------------------

        // - `&Vec3` as `&[f32; 3]`
        // - `&Mat4` as `&[[f32; 4]; 4]`
        impl ::core::convert::AsRef<#wrapped_type> for #struct_name {
            fn as_ref(&self) -> &#wrapped_type {
                &self.#member_name
            }
        }

        // - `&[f32; 3]` as `&Vec3`
        // - `&[[f32; 4]; 4]` as `&Mat4`
        impl ::core::convert::AsRef<#struct_name> for #wrapped_type {
            fn as_ref(&self) -> &#struct_name {
                // SAFETY: `repr(transparent)` on vector and matrix structs makes this conversion guaranteed
                unsafe { ::core::mem::transmute(self) }
            }
        }

        // - `&mut Vec3` as `&mut [f32; 3]`
        // - `&mut Mat4` as `&mut [[f32; 4]; 4]`
        impl ::core::convert::AsMut<#wrapped_type> for #struct_name {
            fn as_mut(&mut self) -> &mut #wrapped_type {
                &mut self.#member_name
            }
        }

        // - `&mut [f32; 3]` as `&mut Vec3`
        // - `&mut [[f32; 4]; 4]` as `&mut Mat4`
        impl ::core::convert::AsMut<#struct_name> for #wrapped_type {
            fn as_mut(&mut self) -> &mut #struct_name {
                // SAFETY: `repr(transparent)` on vector and matrix structs makes this conversion guaranteed
                unsafe { ::core::mem::transmute(self) }
            }
        }

        // -------------------------------------------------------------------------------------
        // Owned conversions (copies)
        // -------------------------------------------------------------------------------------

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

        // - `&[f32; 3]` -> `Vec3`
        // - `&[[f32; 4]; 4]` -> `Mat4`
        impl<'a> ::core::convert::From<&'a #wrapped_type> for #struct_name {
            fn from(value: &#wrapped_type) -> Self {
                // Clone to owned variant (this is a reference to an array, not a slice; cloning gives us a fresh copy
                // of the entire array through that reference)
                Self::from(value.clone())
            }
        }

        // - `&Vec3` -> `[f32; 3]`
        // - `&Mat4` -> `[[f32; 4]; 4]`
        impl<'a> ::core::convert::From<&'a #struct_name> for #wrapped_type {
            fn from(value: &#struct_name) -> Self {
                Self::from(value.clone())
            }
        }
    }
}

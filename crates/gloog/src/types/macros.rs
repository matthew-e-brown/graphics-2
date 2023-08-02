/// Declares a Rust enum with fields that map to specific [`GLenum`] values.
///
/// Enums are automatically declared with:
///
/// - Implementations for [`From<$name> for GLenum`][From] and [`TryFrom<GLenum> for $name`][TryFrom].
/// - Derives with [`Clone`][macro@Clone], [`Copy`][macro@Copy], [`PartialEq`][macro@PartialEq], [`Eq`][macro@Eq], and
///   [`Hash`].
/// - A [`Debug`][core::fmt::Debug] implementation.
///
/// # Syntax
///
/// Simply declare a regular enum, but with additional "match arms" for the OpenGL names. Exclude the `GL_` prefix on
/// the OpenGL names.
///
/// ```
/// gl_enum! {
///     /// Acceptable values for [buffer][Buffer] binding targets.
///
///     pub enum BufferType {
///         /// Buffer target for vertex attributes.
///         ArrayBuffer => ARRAY_BUFFER,
///
///         /* -- snip -- */
///
///         /// Buffer target for uniform block storage.
///         UniformBuffer => UNIFORM_BUFFER,
///     }
/// }
/// ```
///
/// [`GLenum`]: gl::types::GLenum
macro_rules! gl_enum {
    // Empty cases to stop errors before typing any of the variants
    // -----------------------------------------------------------------------------------------------------------------
    () => ();
    (
        $(#[$enum_attrs:meta])*
        $vis:vis enum $enum_name:ident {}
    ) => {
        $(#[$enum_attrs])*
        $vis enum $enum_name {}
    };
    // Main implementation
    // -----------------------------------------------------------------------------------------------------------------
    (
        $(#[$enum_attrs:meta])*
        $vis:vis enum $enum_name:ident {
            $(
                $(#[$field_attrs:meta])*
                $field_name:ident => $gl_name:ident
            ),*$(,)?
        }
    ) => {
        $(#[$enum_attrs])*
        #[derive(Clone, Copy, PartialEq, Eq, Hash)]
        $vis enum $enum_name {
            $(
                $(#[$field_attrs])*
                $field_name
            ),*
        }

        impl ::core::convert::From<$enum_name> for ::gl::types::GLenum {
            fn from(value: $enum_name) -> Self {
                match value {
                    $( $enum_name::$field_name => gl::$gl_name ),*
                }
            }
        }

        impl ::core::convert::TryFrom<::gl::types::GLenum> for $enum_name {
            type Error = crate::errors::EnumConversionError;

            fn try_from(value: ::gl::types::GLenum) -> Result<Self, Self::Error> {
                match value {
                    $( gl::$gl_name => Ok(Self::$field_name), )*
                    other => Err(Self::Error::new(other, stringify!($enum_name))),
                }
            }
        }

        impl ::core::fmt::Debug for $enum_name {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                f.write_str(match self {
                    $ ($enum_name::$field_name => stringify!($field_name), )*
                })
            }
        }
    };
}


/// Declares a Rust struct that wraps a [`GLbitfield`] value to be used for passing bit-flags.
///
/// Structs are automatically declared with:
///
/// - All bitwise operations, including a bitwise implementation of [`Not`][::core::ops::Not].
/// - Derives with [`Clone`][macro@Clone], [`Copy`][macro@Copy], [`PartialEq`][macro@PartialEq], [`Eq`][macro@Eq], and
///   [`Hash`].
/// - A [`Debug`][core::fmt::Debug] implementation.
///
/// # Syntax
///
/// Declare a struct, but instead of any fields, declare public constants for the OpenGL flags. Exclude the `GL_`
/// prefixes.
///
/// ```
/// gl_bitfield! {
///     pub struct BufferMask {
///         pub const COLOR = COLOR_BUFFER_BIT;
///         pub const DEPTH = DEPTH_BUFFER_BIT;
///         pub const STENCIL = STENCIL_BUFFER_BIT;
///     }
/// }
/// ```
///
/// [`GLbitfield`]: gl::types::GLbitfield
macro_rules! gl_bitfield {
    () => ();
    // -----------------------------------------------------------------------------------------------------------------
    (
        $(#[$struct_attrs:meta])*
        $vis:vis struct $struct_name:ident {
            $(
                $(#[$const_attrs:meta])*
                pub const $const_name:ident = $gl_name:ident;
            )*
        }
    ) => {
        $(#[$struct_attrs])*
        #[repr(transparent)]
        #[derive(Clone, Copy, PartialEq, Eq, Hash)]
        $vis struct $struct_name(::gl::types::GLbitfield);

        impl $struct_name {
            $(
                $(#[$const_attrs])*
                pub const $const_name: $struct_name = $struct_name(gl::$gl_name);
            )*

            #[doc="Returns a set of all defined flags."]
            pub const fn all() -> Self {
                Self( $(Self::$const_name.0|)* 0 )
            }

            #[doc="Returns an empty set of flags."]
            pub const fn none() -> Self {
                Self( 0 )
            }

            #[doc="Returns `true` if every one of `other`'s flags are present in `self`."]
            pub const fn contains(&self, other: Self) -> bool {
                // all of `other` is in `self` if masking `self` to only contain `other`'s bits gives us `other`.
                (self.0 & other.0) == other.0
            }
        }


        impl ::core::convert::From<$struct_name> for ::gl::types::GLbitfield {
            fn from(value: $struct_name) -> Self {
                value.0
            }
        }

        impl ::core::convert::TryFrom<::gl::types::GLbitfield> for $struct_name {
            type Error = crate::errors::BitFieldConversionError;

            fn try_from(value: ::gl::types::GLbitfield) -> Result<Self, Self::Error> {
                // Mask `value` to only contain the bits from this bitfield
                let truncated = value & <$struct_name as Into<::gl::types::GLbitfield>>::into(Self::all());
                // If nothing was lost, the passed value was a valid bitfield for this type
                if truncated == value {
                    Ok(Self(value))
                } else {
                    Err(Self::Error::new(value, stringify!($struct_name)))
                }
            }
        }


        impl ::core::fmt::Debug for BufferMask {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                // Take a list of all the possible flags as strings, check if self has each of them, then join together
                // separated by `|`.
                let fields = [ $( (Self::$const_name, stringify!($const_name)), )* ]
                    .into_iter()
                    .filter_map(|(flag, s)| if self.contains(flag) { Some(s) } else { None })
                    .fold(String::new(), |mut acc, cur| {
                        // FIXME: once `intersperse` is stable, use that instead of fold
                        acc.reserve(cur.len() + 1);
                        acc.push_str(cur);
                        acc.push('|');
                        acc
                    });
                let fields = fields.trim_end_matches('|');

                f.debug_tuple(stringify!($struct_name))
                    .field(&format_args!("{fields}"))
                    .finish()
            }
        }


        impl ::core::ops::Not for $struct_name {
            type Output = $struct_name;

            fn not(self) -> Self::Output {
                // negate self, but limit its bits to only contain valid flags
                Self((!self.0) & Self::all().0)
            }
        }

        impl<'a> ::core::ops::Not for &'a $struct_name {
            type Output = $struct_name;

            fn not(self) -> Self::Output {
                !*self
            }
        }

        impl<'a> ::core::ops::Not for &'a mut $struct_name {
            type Output = $struct_name;

            fn not(self) -> Self::Output {
                !*self
            }
        }

        // cspell:ignore bitor bitand bitxor
        gl_bitfield!(impl BitOr for $struct_name, bitor);
        gl_bitfield!(impl BitAnd for $struct_name, bitand);
        gl_bitfield!(impl BitXor for $struct_name, bitxor);

        gl_bitfield!(impl BitOrAssign (assign) for $struct_name, bitor_assign);
        gl_bitfield!(impl BitAndAssign (assign) for $struct_name, bitand_assign);
        gl_bitfield!(impl BitXorAssign (assign) for $struct_name, bitxor_assign);
    };
    // -----------------------------------------------------------------------------------------------------------------
    (impl $op_trait:ident for $struct_name:ident, $op_func:ident) => {
        // owned ⇄ owned
        impl ::core::ops::$op_trait<$struct_name> for $struct_name {
            type Output = $struct_name;

            fn $op_func(self, rhs: $struct_name) -> Self::Output {
                $struct_name(<::gl::types::GLbitfield>::$op_func(self.0, rhs.0))
            }
        }

        // ref ⇄ ref
        impl<'lhs, 'rhs> ::core::ops::$op_trait<&'rhs $struct_name> for &'lhs $struct_name {
            type Output = $struct_name;

            fn $op_func(self, rhs: &'rhs $struct_name) -> Self::Output {
                $struct_name(<::gl::types::GLbitfield>::$op_func(self.0, rhs.0))
            }
        }

        // ref ⇄ owned
        impl<'lhs> ::core::ops::$op_trait<$struct_name> for &'lhs $struct_name {
            type Output = $struct_name;

            fn $op_func(self, rhs: $struct_name) -> Self::Output {
                $struct_name(<::gl::types::GLbitfield>::$op_func(self.0, rhs.0))
            }
        }

        // owned ⇄ ref
        impl<'rhs> ::core::ops::$op_trait<&'rhs $struct_name> for $struct_name {
            type Output = $struct_name;

            fn $op_func(self, rhs: &'rhs $struct_name) -> Self::Output {
                $struct_name(<::gl::types::GLbitfield>::$op_func(self.0, rhs.0))
            }
        }
    };
    (impl $op_trait:ident (assign) for $struct_name:ident, $op_func:ident) => {
        // with owned
        impl ::core::ops::$op_trait<$struct_name> for $struct_name {
            fn $op_func(&mut self, rhs: $struct_name) {
                <::gl::types::GLbitfield>::$op_func(&mut self.0, rhs.0);
            }
        }

        // with ref
        impl<'rhs> ::core::ops::$op_trait<&'rhs $struct_name> for $struct_name {
            fn $op_func(&mut self, rhs: &'rhs $struct_name) {
                <::gl::types::GLbitfield>::$op_func(&mut self.0, rhs.0);
            }
        }

        // with mut
        impl<'rhs> ::core::ops::$op_trait<&'rhs mut $struct_name> for $struct_name {
            fn $op_func(&mut self, rhs: &'rhs mut $struct_name) {
                <::gl::types::GLbitfield>::$op_func(&mut self.0, rhs.0);
            }
        }
    };
}


pub(crate) use {gl_bitfield, gl_enum};

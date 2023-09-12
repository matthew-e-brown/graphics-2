/// Declares a transparent wrapper struct around the given inner type.
///
/// Structs are automatically declared with:
///
/// - `repr(transparent)`.
/// - Derives for [`Debug`], [`Clone`], [`Copy`], [`PartialEq`], [`Eq`], and [`Hash`].
/// - Functions for converting to/from raw [`gl::types::_`][gl::types] values.
///
/// [`Hash`]: std::hash::Hash
/// [`Debug`]: std::fmt::Debug
macro_rules! gl_wrapper {
    (
        $(#[$attr:meta])*
        $vis:vis struct $name:ident($inner:ty)$(;)?
    ) => {
        $(#[$attr])*
        #[repr(transparent)]
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
        $vis struct $name($inner);

        impl $name {
            /// Wraps a raw value returned from an OpenGL binding.
            #[allow(unused)]
            pub(crate) fn new(raw: $inner) -> Self {
                Self(raw)
            }

            /// Returns the raw value of this struct, to be passed to the raw OpenGL bindings.
            #[allow(unused)]
            pub(crate) fn raw(&self) -> $inner {
                self.0
            }
        }
    };
}


/// Declares a Rust enum with fields that map to specific [`GLenum`] values.
///
/// Enums are automatically declared with:
///
/// - `repr(u32)`, which matches `GLenum` (which is specifically "unsigned int").
/// - Derives for [`Debug`], [`Clone`], [`Copy`], [`PartialEq`], [`Eq`], and [`Hash`].
/// - Functions for converting to/from raw [`gl::types::_`][gl::types] values.
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
/// [`Hash`]: std::hash::Hash
/// [`Debug`]: std::fmt::Debug
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
        #[repr(u32)]
        #[derive(Clone, Copy, PartialEq, Eq, Hash)]
        $vis enum $enum_name {
            $(
                $(#[$field_attrs])*
                $field_name = gl::$gl_name,
            )*
        }

        impl $enum_name {
            #[allow(unused)]
            pub(crate) const fn raw(&self) -> gl::types::GLenum {
                // SAFETY: enums are `repr(u32)`:
                // https://doc.rust-lang.org/reference/items/enumerations.html#pointer-casting
                unsafe { *(self as *const Self as *const gl::types::GLenum) }
            }

            #[allow(unused)]
            pub(crate) const fn from_raw(value: gl::types::GLenum) -> Option<Self> {
                match value {
                    $( gl::$gl_name => Some(Self::$field_name), )*
                    _ => None,
                }
            }
        }

        impl std::fmt::Debug for $enum_name {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
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
/// - All bitwise operations, including a bitwise implementation of [`Not`][std::ops::Not].
/// - Derives for [`Debug`], [`Clone`], [`Copy`], [`PartialEq`], [`Eq`], and [`Hash`].
/// - Functions for converting to/from raw [`gl::types::_`][gl::types] values.
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
/// [`Hash`]: std::hash::Hash
/// [`Debug`]: std::fmt::Debug
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
        $vis struct $struct_name(gl::types::GLbitfield);

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

        impl $struct_name {
            #[allow(unused)]
            pub(crate) const fn raw(&self) -> gl::types::GLbitfield {
                self.0
            }

            #[allow(unused)]
            pub(crate) const fn from_raw(value: gl::types::GLbitfield) -> Option<Self> {
                // Mask `value` to only include bytes from this bitfield
                let masked = value & Self::all().raw();
                // If we didn't lose any bits doing so, then this is a proper set of our bits
                if masked == value {
                    Some(Self(value))
                } else {
                    None
                }
            }
        }

        impl std::fmt::Debug for $struct_name {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                // Take a list of all the possible flags as strings, check if self has each of them, then join together
                // separated by `|`.
                let fields = [ $( (Self::$const_name, stringify!($const_name)), )* ]
                    .into_iter()
                    .filter_map(|(flag, s)| if self.contains(flag) { Some(s) } else { None })
                    .collect::<Vec<&str>>()
                    .join("|");
                f.debug_tuple(stringify!($struct_name)).field(&fields).finish()
            }
        }


        impl std::ops::Not for $struct_name {
            type Output = $struct_name;

            fn not(self) -> Self::Output {
                // negate self, but limit its bits to only contain valid flags
                Self((!self.0) & Self::all().0)
            }
        }

        impl<'a> std::ops::Not for &'a $struct_name {
            type Output = $struct_name;

            fn not(self) -> Self::Output {
                !*self
            }
        }

        impl<'a> std::ops::Not for &'a mut $struct_name {
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
        impl std::ops::$op_trait<$struct_name> for $struct_name {
            type Output = $struct_name;

            fn $op_func(self, rhs: $struct_name) -> Self::Output {
                $struct_name(<gl::types::GLbitfield>::$op_func(self.0, rhs.0))
            }
        }

        // ref ⇄ ref
        impl<'lhs, 'rhs> std::ops::$op_trait<&'rhs $struct_name> for &'lhs $struct_name {
            type Output = $struct_name;

            fn $op_func(self, rhs: &'rhs $struct_name) -> Self::Output {
                $struct_name(<gl::types::GLbitfield>::$op_func(self.0, rhs.0))
            }
        }

        // ref ⇄ owned
        impl<'lhs> std::ops::$op_trait<$struct_name> for &'lhs $struct_name {
            type Output = $struct_name;

            fn $op_func(self, rhs: $struct_name) -> Self::Output {
                $struct_name(<gl::types::GLbitfield>::$op_func(self.0, rhs.0))
            }
        }

        // owned ⇄ ref
        impl<'rhs> std::ops::$op_trait<&'rhs $struct_name> for $struct_name {
            type Output = $struct_name;

            fn $op_func(self, rhs: &'rhs $struct_name) -> Self::Output {
                $struct_name(<gl::types::GLbitfield>::$op_func(self.0, rhs.0))
            }
        }
    };
    (impl $op_trait:ident (assign) for $struct_name:ident, $op_func:ident) => {
        // with owned
        impl std::ops::$op_trait<$struct_name> for $struct_name {
            fn $op_func(&mut self, rhs: $struct_name) {
                <gl::types::GLbitfield>::$op_func(&mut self.0, rhs.0);
            }
        }

        // with ref
        impl<'rhs> std::ops::$op_trait<&'rhs $struct_name> for $struct_name {
            fn $op_func(&mut self, rhs: &'rhs $struct_name) {
                <gl::types::GLbitfield>::$op_func(&mut self.0, rhs.0);
            }
        }

        // with mut
        impl<'rhs> std::ops::$op_trait<&'rhs mut $struct_name> for $struct_name {
            fn $op_func(&mut self, rhs: &'rhs mut $struct_name) {
                <gl::types::GLbitfield>::$op_func(&mut self.0, rhs.0);
            }
        }
    };
}


pub(super) use {gl_bitfield, gl_enum, gl_wrapper};

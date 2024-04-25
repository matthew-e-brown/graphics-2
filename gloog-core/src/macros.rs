// Rust doc tests cannot see private items, so all of the code-blocks in these doc-comments are marked with `ignore`.

/// A convenience macro for converting Rust-native expressions into [GL-types][gl::types] with [`TryFrom`] (or really
/// anything that needs to be `TryFrom`'d).
///
/// Because the GL-types are all type aliases, many calls to this macro will probably get optimized away. As in, since
/// a `GLint` is just an alias for `i32`, something like `convert!(some_i32, GLint, "buffer count")` will just become
/// `<i32>::TryFrom::<i32>::try_from(some_i32)`, which is just gonna be a no-op.
///
/// Some conversions are trivial enough that they don't really need this macro; however, I find that the names like
/// "`GLsizei`" often seem ambiguous as far as which Rust types they're referring to. So, even if a conversion _is_
/// trivial enough so as to not to need this macro, I'll usually still use it just to be safe.
///
/// The most common use-case for this macro is just to ease the repetitiveness of constantly trying to `try_into` a
/// larger, Rust type (like `usize` for a buffer length) into an OpenGL type (like `GLsizei`, which many commands expect
/// instead of unsigned 64-bit sizes).
///
/// # Syntax
///
/// Pass three comma separated arguments to this macro:
///
/// 1.  The expression to convert;
/// 2.  The type to attempt to convert it into; and
/// 3.  A string which names what sort of value is being converted, to be printed with `.expect` if the conversion
///     fails.
///
/// For example, something like this:
///
/// ```ignore
/// let num_buffers = convert!(buffers.len(), GLsizei, "buffer creation count");
/// ```
///
/// The third parameter is normally given simply as a name for the value being converted. This will cause `.expect` to
/// print out "`[name] should fit into a '[type]'`". For example:
///
/// ```ignore
/// let buf_size = convert!(usize::MAX, GLsizei, "buffer size") // -> "buffer size should fit into a 'GLsizei'"
/// ```
///
/// A unique message can be provided by prefixing the third parameter with `msg:`, at which point the string is passed
/// to `.expect` exactly as-is.
///
/// ```ignore
/// let size = convert!(buff.len(), GLsizei, msg: "your buffer is too large!!");
/// ```
macro_rules! convert {
    ($src:expr, $into:ty, $src_name:expr$(,)?) => {
        $crate::convert!($src, $into, msg: concat!($src_name, " should fit into a `", stringify!($into), "`"))
    };
    ($src:expr, $into:ty, msg: $err_msg:expr$(,)?) => {
        <$into>::try_from($src).expect($err_msg)
    };
}


/// Declares a transparent wrapper struct around the given inner type.
///
/// Structs are automatically declared with:
///
/// - `repr(transparent)`.
/// - Derives for [`Debug`], [`Clone`], [`Copy`], [`PartialEq`], [`Eq`], and [`Hash`].
/// - Functions for converting to/from raw values.
///
/// [`Hash`]: std::hash::Hash
/// [`Debug`]: std::fmt::Debug
macro_rules! gl_newtype {
    (
        $(#[$attr:meta])*
        $vis:vis struct $name:ident($inner:ty)$(;)?
    ) => {
        $(#[$attr])*
        #[repr(transparent)]
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
        // NB: `pub(crate)` is hardcoded because (a) we don't need public access due to `into_raw` and `from_raw`
        // functions, and (b) without it, crate functions can't match on the inner values of generated types.
        $vis struct $name(pub(crate) $inner);

        impl $name {
            /// Wraps a raw value (usually called a "name" or "location", depending on the context) as a newtype struct.
            ///
            /// # Safety
            ///
            /// Callers should be sure that the given value is valid.
            pub const unsafe fn from_raw_unchecked(value: $inner) -> Self {
                Self(value)
            }

            /// Returns the underlying value of this wrapper struct, for interop with raw OpenGL bindings.
            pub const fn into_raw(self) -> $inner {
                self.0
            }
        }
    };
}


/// Declares a Rust enum with fields that map to specific `GLenum` values.
///
/// Enums are automatically declared with:
///
/// - `repr(u32)`, which matches `GLenum`.
/// - Derives for [`Debug`], [`Clone`], [`Copy`], [`PartialEq`], [`Eq`], and [`Hash`].
/// - Functions for converting to/from raw values.
///
/// # Syntax
///
/// Simply declare a regular enum, but with additional "match arms" for the OpenGL names. Exclude the `GL_` prefix on
/// the OpenGL names.
///
/// ```ignore
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
/// Additional hardcoded `u32` values can be specified by prefixing them with `(literal)`. `(literal)` must be used
/// instead of `#[literal]` due to a limitation in how declarative macros handle ambiguities. Similarly, all literals
/// must be at the start of the list. Additionally, `(literal)` must come before any other attributes (i.e.
/// doc-comments).
///
/// [`Hash`]: std::hash::Hash
/// [`Debug`]: std::fmt::Debug
macro_rules! gl_enum {
    // Empty cases to stop errors before typing any of the variants
    // -----------------------------------------------------------------------------------------------------------------
    () => ();
    (
        $(#[$enum_attrs:meta])*
        $vis:vis enum $enum_name:ident {}
    ) => {
        $(#[$enum_attrs])*
        #[repr(u32)]
        #[derive(Clone, Copy, PartialEq, Eq, Hash)]
        $vis enum $enum_name {}
    };
    // Main implementation
    // -----------------------------------------------------------------------------------------------------------------
    (
        $(#[$enum_attrs:meta])*
        $vis:vis enum $enum_name:ident {
            $(
                (literal)
                $(#[$lit_attrs:meta])*
                $lit_name:ident => $lit:literal
            ),*$(,)?
            $(
                $(#[$field_attrs:meta])*
                $field_name:ident => $gl_name:ident
            ),*$(,)?
        }
    ) => {
        $(#[$enum_attrs])*
        #[repr(u32)] // NB: GLenum is hardcoded to be u32. GLenum is used elsewhere as a type name only for clarity.
        #[derive(Clone, Copy, PartialEq, Eq, Hash)]
        $vis enum $enum_name {
            // Any literals come first;
            $(
                $(#[$lit_attrs])*
                $lit_name = $lit,
            )*
            // Followed by actual OpenGL mappings.
            $(
                $(#[$field_attrs])*
                $field_name = crate::raw::$gl_name,
            )*
        }

        impl $enum_name {
            /// Returns the raw value of this enum as understood by OpenGL.
            pub const fn into_raw(&self) -> crate::raw::types::GLenum {
                // SAFETY: This enum is `repr(u32)`, which is what `GLenum` is.
                unsafe { *(self as *const Self as *const u32) }

                // NB: This is the recommended way to access enum discriminants for primitive-represented enums.
                // https://doc.rust-lang.org/reference/items/enumerations.html#pointer-casting
            }

            /// Converts the given value into an instance of this enum by matching on all possible values.
            pub const fn from_raw(value: crate::raw::types::GLenum) -> Option<Self> {
                match value{
                    $( $lit => Some(Self::$lit_name), )*
                    $( crate::raw::$gl_name => Some(Self::$field_name), )*
                    _ => None,
                }
            }

            /// Converts the given value into an instance of this enum without checking if it matches a specific enum
            /// variant first.
            ///
            /// Callers should be sure that the given value is in fact a valid OpenGL enum for the various functions
            /// this Rust enum claims to map to. For example, just because en enum has `UnsignedByte` as a variant
            /// doesn't mean that `GL_FLOAT` is valid; check which functions are associated to this enum before
            /// performing a "cast" like this.
            pub const unsafe fn from_raw_unchecked(value: crate::raw::types::GLenum) -> Self {
                // SAFETY: This enum is `repr(u32)`, which is what `GLenum` is.
                unsafe { *(&value as *const crate::raw::types::GLenum as *const Self) }
            }
        }

        impl std::fmt::Debug for $enum_name {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.write_str(match self {
                    $($enum_name::$lit_name => stringify!($lit_name), )*
                    $($enum_name::$field_name => stringify!($field_name), )*
                })
            }
        }
    };
}


/// Declares a Rust struct that wraps a `GLbitfield` value to be used for passing bit-flags.
///
/// Structs are automatically declared with:
///
/// - All bitwise operations, including a bitwise implementation of [`Not`][std::ops::Not].
/// - Derives for [`Debug`], [`Clone`], [`Copy`], [`PartialEq`], [`Eq`], and [`Hash`].
/// - Functions for converting to/from raw values.
///
/// # Syntax
///
/// Declare a struct, but instead of any fields, declare public constants for the OpenGL flags. Exclude the `GL_`
/// prefixes.
///
/// ```ignore
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
        $vis struct $struct_name(u32);

        // The actual values of this bitfield; the associated constants.
        impl $struct_name {
            $(
                $(#[$const_attrs])*
                pub const $const_name: $struct_name = $struct_name(crate::raw::$gl_name);
            )*
        }

        // Function implementations.
        impl $struct_name {
            /// Returns a set of all defined flags.
            pub const fn all() -> Self {
                Self( $(Self::$const_name.0|)* 0 )
            }

            /// Returns an empty set of flags.
            pub const fn none() -> Self {
                Self( 0 )
            }

            /// Returns `true` if every one of `other`'s flags are present in `self`.
            pub const fn contains(&self, other: Self) -> bool {
                // all of `other` is in `self` if masking `self` to only contain `other`'s bits gives us `other`.
                (self.0 & other.0) == other.0
            }

            /// Returns the underlying value of this bitfield as understood by OpenGL.
            pub const fn into_raw(&self) -> crate::raw::types::GLbitfield {
                self.0
            }

            /// Converts a raw `GLbitfield` (a [`u32`]) into this bitfield struct.
            ///
            /// Returns `None` if the given value does not contain a valid set of bits for this particular bitfield.
            pub const fn from_raw(value: crate::raw::types::GLbitfield) -> Option<Self> {
                // Mask `value` to only include the bits that're in this bitfield
                let masked = value & Self::all().into_raw();
                // If we didn't lose any bits doing so, then this is a valid set of bits for this type.
                if masked == value {
                    Some(Self(value))
                } else {
                    None
                }
            }

            /// Converts a raw `GLbitfield` (a [`u32`]) into this bitfield struct without checking whether or not its
            /// bits are valid first.
            ///
            /// Callers should ensure that creating this value will not result in sending any erroneous bitflags to
            /// OpenGL function calls, since some commands are defined such that doing so is an error. For example, if a
            /// function expects a bitfield with two flags, passing a third may be an error (i.e., if a command expects
            /// `00`, `01`, `10`, or `11`, passing `100` may not interpreted as `00`, but may instead generate an
            /// error).
            pub const unsafe fn from_raw_unchecked(value: crate::raw::types::GLbitfield) -> Self {
                Self(value)
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
                f.write_str(concat!(stringify!($struct_name), "("))?;
                f.write_str(&fields)?;
                f.write_str(")")?;
                Ok(())
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
                $struct_name(u32::$op_func(self.0, rhs.0))
            }
        }

        // ref ⇄ ref
        impl<'lhs, 'rhs> std::ops::$op_trait<&'rhs $struct_name> for &'lhs $struct_name {
            type Output = $struct_name;

            fn $op_func(self, rhs: &'rhs $struct_name) -> Self::Output {
                $struct_name(u32::$op_func(self.0, rhs.0))
            }
        }

        // ref ⇄ owned
        impl<'lhs> std::ops::$op_trait<$struct_name> for &'lhs $struct_name {
            type Output = $struct_name;

            fn $op_func(self, rhs: $struct_name) -> Self::Output {
                $struct_name(u32::$op_func(self.0, rhs.0))
            }
        }

        // owned ⇄ ref
        impl<'rhs> std::ops::$op_trait<&'rhs $struct_name> for $struct_name {
            type Output = $struct_name;

            fn $op_func(self, rhs: &'rhs $struct_name) -> Self::Output {
                $struct_name(u32::$op_func(self.0, rhs.0))
            }
        }
    };
    (impl $op_trait:ident (assign) for $struct_name:ident, $op_func:ident) => {
        // with owned
        impl std::ops::$op_trait<$struct_name> for $struct_name {
            fn $op_func(&mut self, rhs: $struct_name) {
                u32::$op_func(&mut self.0, rhs.0);
            }
        }

        // with ref
        impl<'rhs> std::ops::$op_trait<&'rhs $struct_name> for $struct_name {
            fn $op_func(&mut self, rhs: &'rhs $struct_name) {
                u32::$op_func(&mut self.0, rhs.0);
            }
        }

        // with mut
        impl<'rhs> std::ops::$op_trait<&'rhs mut $struct_name> for $struct_name {
            fn $op_func(&mut self, rhs: &'rhs mut $struct_name) {
                u32::$op_func(&mut self.0, rhs.0);
            }
        }
    };
}


pub(super) use {convert, gl_bitfield, gl_enum, gl_newtype};

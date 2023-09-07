mod vec2;
mod vec3;
mod vec4;

pub use vec2::*;
pub use vec3::*;
pub use vec4::*;


/// Implements the common, core components of a column vector of floats.
///
/// Note to self: vectors using this **must** be `repr(C)`!
macro_rules! impl_vector_basics {
    ($name:ident, $inner:ty, $count:literal, { $($n:literal: $x:ident),+ }) => {
        impl ::core::default::Default for $name {
            fn default() -> Self {
                Self::zeroed()
            }
        }

        // =============================================================================================================
        // Operators
        // =============================================================================================================

        $crate::operator!(+ (commutative) #[inline] |a: $name, b: $inner| -> $name { $name { $($x: a.$x + b),* } });
        $crate::operator!(- (commutative) #[inline] |a: $name, b: $inner| -> $name { $name { $($x: a.$x - b),* } });
        $crate::operator!(* (commutative) #[inline] |a: $name, b: $inner| -> $name { $name { $($x: a.$x * b),* } });
        $crate::operator!(/ (commutative) #[inline] |a: $name, b: $inner| -> $name { $name { $($x: a.$x / b),* } });
        $crate::operator!(+= #[inline] |a: &mut $name, b: $inner| { $(a.$x += b;)* });
        $crate::operator!(-= #[inline] |a: &mut $name, b: $inner| { $(a.$x -= b;)* });
        $crate::operator!(*= #[inline] |a: &mut $name, b: $inner| { $(a.$x *= b;)* });
        $crate::operator!(/= #[inline] |a: &mut $name, b: $inner| { $(a.$x /= b;)* });

        $crate::operator!(+ #[inline] |a: $name, b: $name| -> $name { $name { $($x: a.$x + b.$x),* } });
        $crate::operator!(- #[inline] |a: $name, b: $name| -> $name { $name { $($x: a.$x - b.$x),* } });
        $crate::operator!(+= #[inline] |a: &mut $name, b: $name| { $(a.$x += b.$x;)* });
        $crate::operator!(-= #[inline] |a: &mut $name, b: $name| { $(a.$x -= b.$x;)* });

        // =============================================================================================================
        // Core and mathematic implementations
        // =============================================================================================================

        impl $name {
            /// Creates a new vector.
            #[inline]
            pub const fn new($($x: $inner),*) -> Self {
                Self { $($x),* }
            }

            /// Computes the dot product between this and another vector.
            ///
            /// This vector is the left-hand operand.
            #[inline]
            pub fn dot(&self, rhs: &$name) -> $inner {
                $crate::strip_plus!($(+ (self.$x * rhs.$x))*)
            }

            /// Computes the magnitude of this vector, squared.
            ///
            /// The omission of the square-root is helpful for speed either when the squared magnitude is the end goal,
            /// to prevent re-squaring, or when comparing two vectors' lengths (a > b implies a² > b², and vice versa).
            #[inline]
            pub fn mag_sq(&self) -> $inner {
                $crate::strip_plus!($(+ self.$x * self.$x)*)
            }

            /// Computes the magnitude of this vector.
            #[inline]
            pub fn mag(&self) -> $inner {
                self.mag_sq().sqrt()
            }

            /// Computes a new vector with the same direction as this one, but with a magnitude of one.
            #[inline]
            pub fn norm(&self) -> $name {
                self / self.mag()
            }

            /// Computes the vector projection of this vector onto another.
            pub fn project(&self, onto: &$name) -> $name {
                onto * (self.dot(onto) / onto.mag_sq())
            }

            /// Computes the vector rejection of this vector from another.
            ///
            /// The resulting vector will be perpendicular to `from` in the direction of `self`.
            #[inline]
            pub fn reject(&self, from: &$name) -> $name {
                self - self.project(from)
            }

            /// Gets a pointer to the first element of this vector.
            pub const fn as_ptr(&self) -> *const $inner {
                // SAFETY: `repr(C)` guarantees that the first element has the same address as `self`, so we can safely
                // cast `*const Vec` to `*const f32`.
                self as *const $name as *const $inner
            }

            /// Gets a mutable pointer to the first element of this vector.
            pub fn as_mut_ptr(&mut self) -> *mut $inner {
                // SAFETY: see `as_ptr` above.
                self as *mut $name as *mut $inner
            }

            /// Gets a reference to this vector as a slice of bytes.
            pub fn as_bytes(&self) -> &[u8] {
                bytemuck::bytes_of(self)
            }

            /// Converts this vector to an array of bytes.
            pub fn to_bytes(self) -> [u8; ::core::mem::size_of::<$inner>() * $count] {
                // SAFETY: `self`'s length is always guaranteed to be exactly the same as `$count × sizeof<$inner>`; so
                // slice->array will never fail.
                unsafe { self.as_bytes().try_into().unwrap_unchecked() }
            }
        }

        // =============================================================================================================
        // Utility implementations
        // =============================================================================================================

        // Conversions into inner type
        // ----------------------------------------------------------------------------------------

        impl ::core::convert::From<$name> for [$inner; $count] {
            fn from(value: $name) -> Self {
                [ $(value.$x),* ]
            }
        }

        impl ::core::convert::AsRef<[$inner; $count]> for $name {
            fn as_ref(&self) -> &[$inner; $count] {
                // SAFETY: `repr(C)` guarantees that this struct is always identical to `[T; n]`, at least for the types
                // our vectors use.
                unsafe { ::core::mem::transmute::<&$name, &[$inner; $count]>(self) }
            }
        }

        impl ::core::convert::AsMut<[$inner; $count]> for $name {
            fn as_mut(&mut self) -> &mut [$inner; $count] {
                // SAFETY: `repr(C)` guarantees that this struct is always identical to `[T; n]`, at least for the types
                // our vectors use.
                unsafe { ::core::mem::transmute::<&mut $name, &mut [$inner; $count]>(self) }
            }
        }

        impl ::core::borrow::Borrow<[$inner; $count]> for $name {
            fn borrow(&self) -> &[$inner; $count] {
                self.as_ref()
            }
        }

        impl ::core::borrow::BorrowMut<[$inner; $count]> for $name {
            fn borrow_mut(&mut self) -> &mut [$inner; $count] {
                self.as_mut()
            }
        }

        impl $name {
            /// Gets a reference to this vector, casted to an array representation.
            ///
            /// This is equivalent to calling [`as_ref`][core::convert::AsRef::as_ref], but without the need for type
            /// annotations.
            #[inline]
            pub fn as_array(&self) -> &[$inner; $count] {
                self.as_ref()
            }

            /// Gets a mutable reference to this vector, casted to an array representation.
            ///
            /// This is equivalent to calling [`as_mut`][core::convert::AsMut::as_mut], but without the need for type
            /// annotations.
            #[inline]
            pub fn as_mut_array(&mut self) -> &mut [$inner; $count] {
                self.as_mut()
            }
        }

        // Conversions from inner type
        // ----------------------------------------------------------------------------------------

        impl ::core::convert::From<[$inner; $count]> for $name {
            fn from(value: [$inner; $count]) -> Self {
                $name {
                    $($x: value[$n]),*
                }
            }
        }

        impl ::core::convert::TryFrom<&[$inner]> for $name {
            type Error = ::core::array::TryFromSliceError;

            fn try_from(value: &[$inner]) -> Result<Self, Self::Error> {
                let value: [$inner; $count] = value.try_into()?;
                Ok(value.into())
            }
        }

        impl ::core::convert::AsRef<$name> for [$inner; $count] {
            fn as_ref(&self) -> &$name {
                // SAFETY: `repr(C)` guarantees that this array is always identical our struct
                unsafe { ::core::mem::transmute::<&[$inner; $count], &$name>(self) }
            }
        }

        impl ::core::convert::AsMut<$name> for [$inner; $count] {
            fn as_mut(&mut self) -> &mut $name {
                // SAFETY: `repr(C)` guarantees that this array is always identical our struct
                unsafe { ::core::mem::transmute::<&mut [$inner; $count], &mut $name>(self) }
            }
        }

        impl ::core::borrow::Borrow<$name> for [$inner; $count] {
            fn borrow(&self) -> &$name {
                self.as_ref()
            }
        }

        impl ::core::borrow::BorrowMut<$name> for [$inner; $count] {
            fn borrow_mut(&mut self) -> &mut $name {
                self.as_mut()
            }
        }

        // Indexing
        // ----------------------------------------------------------------------------------------

        // Indexing is implemented for any type that an array of `T` (`f32`) can be indexed with; this automatically
        // gives support for `&vec4[1..2]` to get a `&[f32]`.
        //
        // Because vectors aren't 2D or anything, we don't have to worry about conflicting blanket implementations like
        // we do for matrices.

        impl<I> ::core::ops::Index<I> for $name
        where
            [$inner; $count]: ::core::ops::Index<I>,
        {
            type Output = <[$inner; $count] as ::core::ops::Index<I>>::Output;

            #[inline]
            fn index(&self, index: I) -> &Self::Output {
                self.as_array().index(index)
            }
        }

        impl<I> ::core::ops::IndexMut<I> for $name
        where
            [$inner; $count]: ::core::ops::IndexMut<I>,
        {
            #[inline]
            fn index_mut(&mut self, index: I) -> &mut Self::Output {
                self.as_mut_array().index_mut(index)
            }
        }
    };
}


use impl_vector_basics;

mod vec2;
mod vec3;
mod vec4;

use std::ops::Range;

use thiserror::Error;
pub use vec2::*;
pub use vec3::*;
pub use vec4::*;


/// Implements the common, core components of a column vector.
///
/// Notes to self:
/// - This only works on vectors of floats (either `f32` or `f64`) because it uses `1.0 / f` in division operators;
/// - Any vectors using this **must** be `repr(C)`.
macro_rules! impl_vector_basics {
    ($name:ident, $inner:ty, $count:literal ($num_bytes:literal), { $($n:literal: $x:ident),+ }) => {
        impl core::default::Default for $name {
            fn default() -> Self {
                Self::zeroed()
            }
        }

        // =============================================================================================================
        // Operators
        // =============================================================================================================

        $crate::operator!(+ (commutative) #[inline] |a: &$name, b: &$inner| -> $name { $name { $($x: a.$x + b),* } });
        $crate::operator!(- (commutative) #[inline] |a: &$name, b: &$inner| -> $name { $name { $($x: a.$x - b),* } });
        $crate::operator!(* (commutative) #[inline] |a: &$name, b: &$inner| -> $name { $name { $($x: a.$x * b),* } });
        $crate::operator!(/ #[inline] |a: &$name, b: &$inner| -> $name { let b = 1.0 / b; $name { $($x: a.$x * b),* } });
        $crate::operator!(/ #[inline] |a: &$inner, b: &$name| -> $name { $name { $($x: a / b.$x),* } });

        $crate::operator!(+= #[inline] |a: &mut $name, b: &$inner| { $(a.$x += b;)* });
        $crate::operator!(-= #[inline] |a: &mut $name, b: &$inner| { $(a.$x -= b;)* });
        $crate::operator!(*= #[inline] |a: &mut $name, b: &$inner| { $(a.$x *= b;)* });
        $crate::operator!(/= #[inline] |a: &mut $name, b: &$inner| { let b = 1.0 / b; $(a.$x *= b;)* });

        $crate::operator!(+ #[inline] |a: &$name, b: &$name| -> $name { $name { $($x: a.$x + b.$x),* } });
        $crate::operator!(- #[inline] |a: &$name, b: &$name| -> $name { $name { $($x: a.$x - b.$x),* } });

        $crate::operator!(+= #[inline] |a: &mut $name, b: &$name| { $(a.$x += b.$x;)* });
        $crate::operator!(-= #[inline] |a: &mut $name, b: &$name| { $(a.$x -= b.$x;)* });

        $crate::operator!(- #[inline] |a: &$name| -> $name { $name { $($x: -a.$x),* } });

        // =============================================================================================================
        // Core and mathematic implementations
        // =============================================================================================================

        impl $name {
            /// Creates a new vector.
            #[inline]
            pub const fn new($($x: $inner),*) -> Self {
                Self { $($x),* }
            }

            /// Computes the magnitude of this vector.
            #[inline]
            pub fn mag(&self) -> $inner {
                self.mag_sq().sqrt()
            }

            /// Computes the squared magnitude of this vector.
            ///
            /// Omitting the call to [`sqrt`][f32::sqrt] is a useful optimization in several cases, notably when comparing two
            /// vectors' lengths (a > b implies a² > b², and vice versa), or when certain mathematical operations already
            /// require the squared magnitude.
            #[inline]
            pub fn mag_sq(&self) -> $inner {
                self.dot(self)
            }

            /// Computes the dot product between this and another vector.
            #[inline]
            pub fn dot(&self, rhs: &$name) -> $inner {
                $crate::strip_plus!($(+ (self.$x * rhs.$x))*)
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
        }

        // =============================================================================================================
        // Utility implementations
        // =============================================================================================================

        impl $name {
            /// Gets a pointer to the first element of this vector.
            ///
            /// Because this struct is `repr(C)`, this pointer will always be equal to the pointer of the struct itself.
            #[inline]
            pub const fn as_ptr(&self) -> *const $inner {
                &self.x as *const $inner
            }

            /// Interprets this vector as an array of floats.
            #[inline]
            pub const fn as_array(&self) -> &[$inner; $count] {
                let ptr = self.as_ptr().cast();
                // SAFETY: `Self` is `repr(C)`; by Rust's definitions of `repr(C)` and array packing, the two types are
                // identical and we can safely cast between the two. We know that we can dereference without issue
                // because the `&self` we started with is guaranteed to be valid.
                unsafe { &*ptr }
            }

            /// Interprets this vector as raw bytes.
            #[inline]
            pub const fn as_bytes(&self) -> &[u8; $num_bytes] {
                let ptr = self.as_ptr().cast();
                // SAFETY: see `as_array`.
                unsafe { &*ptr }
            }

            /// Gets a mutable pointer to the first element of this vector.
            ///
            /// Because this struct is `repr(C)`, this pointer will always be equal to the pointer of the struct itself.
            #[inline]
            pub fn as_mut_ptr(&mut self) -> *mut $inner {
                &mut self.x as *mut $inner
            }

            /// Interprets this vector as mutable array of floats.
            #[inline]
            pub fn as_mut_array(&mut self) -> &mut [$inner; $count] {
                let ptr = self.as_mut_ptr().cast();
                // SAFETY: see `as_array`.
                unsafe { &mut *ptr }
            }

            /// Interprets this vector as raw, mutable bytes.
            #[inline]
            pub fn as_mut_bytes(&mut self) -> &mut [u8; $num_bytes] {
                let ptr = self.as_mut_ptr().cast();
                // SAFETY: see `as_array`.
                unsafe { &mut *ptr }
            }
        }

        // Conversions to/from inner type
        // ----------------------------------------------------------------------------------------

        impl From<[$inner; $count]> for $name {
            fn from(value: [$inner; $count]) -> Self {
                $name { $($x: value[$n]),* }
            }
        }

        impl From<$name> for [$inner; $count] {
            fn from(value: $name) -> Self {
                *value.as_array()
            }
        }

        // Indexing
        // ----------------------------------------------------------------------------------------

        // Indexing is implemented for any type that an array of `T` (`f32`) can be indexed with; this automatically
        // gives support for `&vec4[1..2]` to get a `&[f32]`.
        //
        // Because vectors aren't 2D or anything, we don't have to worry about conflicting blanket implementations like
        // we do for matrices.

        impl<I: core::slice::SliceIndex<[$inner]>> core::ops::Index<I> for $name {
            type Output = <I as core::slice::SliceIndex<[$inner]>>::Output;

            #[inline]
            fn index(&self, index: I) -> &Self::Output {
                self.as_array().index(index)
            }
        }

        impl<I: core::slice::SliceIndex<[$inner]>> core::ops::IndexMut<I> for $name where {
            #[inline]
            fn index_mut(&mut self, index: I) -> &mut Self::Output {
                self.as_mut_array().index_mut(index)
            }
        }
    };
}

use impl_vector_basics;


#[derive(Error, Debug)]
pub enum ParseVecError {
    #[error("encountered invalid float at range {0:?}")]
    InvalidFloat(Range<usize>),

    #[error("encountered {0} of required {1} vector components")]
    TooFewComponents(u32, u32),

    #[error("encountered more than the required {0} vector components")]
    TooManyComponents(u32),
}

/// Helper function for parsing a vector from a string.
fn parse_vec<const DIM: usize>(s: &str) -> Result<[f32; DIM], ParseVecError> {
    let mut arr = [0.0; DIM];

    // Ignore whitespace at the start and end, just like we do between floats
    let s = s.trim();

    let mut num_idx = Some(0); // current float's starting index
    let mut arr_idx = 0; // index into output array

    // we want to split at either a comma or whitespace; but when we encounter a comma, we also want to eat all
    // whitespace. so we'll loop ourselves instead of using one of Rust's splitter functions. Regex would be a bit too
    // heavy-duty for this.
    for (c_idx, cur_char) in s.char_indices() {
        // If we're currently inside of a float, `f_start` will be Some(idx)
        if let Some(s_idx) = num_idx {
            // We're inside of a float right now.

            // If we manage to make it back here after parsing the DIM'th component, then they managed to find more
            // non-whitespace characters after we parsed the last component.
            if arr_idx >= DIM {
                let d = DIM.try_into().unwrap(); // we know `2 <= DIM <= 4`; safe to unwrap
                return Err(ParseVecError::TooManyComponents(d));
            }

            // If we find a non-whitespace character, then the last character before this was the end of our float. Or,
            // if we hit the end of the string, we need to parse.
            if cur_char == ',' || cur_char.is_whitespace() || c_idx == s.len() - 1 {
                num_idx = None; // no longer inside float

                arr[arr_idx] = s[s_idx..c_idx].parse().map_err(|_| ParseVecError::InvalidFloat(s_idx..c_idx))?;
                arr_idx += 1;
            }
        } else if cur_char != ',' && !cur_char.is_whitespace() {
            // Otherwise, we're between floats; if we found a non-comma, non-whitespace character, that's the start of
            // the next float (and if it isn't a float, `f32::parse` will handle it).
            num_idx = Some(c_idx);
        }
    }

    // If we got to the end without having parsed the DIM'th component, we didn't get enough components.
    if arr_idx < DIM {
        // we know `2 <= DIM <= 4` (and `arr_idx` is less than that), so unwrapping is fine
        let d = DIM.try_into().unwrap();
        let i = arr_idx.try_into().unwrap();
        Err(ParseVecError::TooFewComponents(d, i))
    } else {
        Ok(arr)
    }
}

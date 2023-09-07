mod mat2;
mod mat3;
mod mat4;

pub use mat2::*;
pub use mat3::*;
pub use mat4::*;


/// Implements the common, core components of a matrix of floats.
///
/// This macro has some whack-ass syntax because it needs a whole bunch of specific identifiers, literals, etc., all
/// provided in a specific order with specific nesting to enable proper declarative macro repetition behaviours (e.g.,
/// `new_rm` needs to receive the same identifiers as `new`, but they need to be repeated in a different order).
///
/// Note to self: matrices using this **must** be `repr(C)`!
macro_rules! impl_matrix_basics {
    (
        $name:ident, $inner:ty, $rows:literal * $cols:literal, {
            col_type: $col_type:path,
            // This is hard as hell to read, but it's an array of:
            // ```
            // [
            //      c0/C0/0: [ n00: (0, 0), ... ] / [r1, r2, ...],
            //      ...,
            // ]
            // ```
            // where `c0` and `C0` are the parameter names and parameter generics for that given column in `from_cols`.
            // The `r_` identifiers in the second array are each of the rows that get indexed to form a column in
            // `from_rows`.
            col_order: [$(
                $col_param:ident/$col_generic:ident/$col_idx:literal: [$(
                    $param_n:ident: ($c:literal, $r:literal$(,)?)
                ),*] / [$($fr_ident:ident$(,)?),*]$(,)?
            ),*],
            // Identifiers and generics used for `from_rows`
            fr_params: [$(
                $row_param:ident/$row_generic:ident$(,)?
            ),*],
            // This one is no easier to read, but it's an array of:
            // ```
            // [ ..., [ n20 -> n02, ... ], ..., ]
            // ```
            // which maps entries of the matrix row-major to column-major based on the *position* of the element.
            // Just... look at an example. lol.
            rm_mapping: [$(
                [$(
                    $rm_param_src:ident -> $rm_param_dst:ident$(,)?
                ),*]$(,)?
            ),*]$(,)?
        }
    ) => {
        impl ::core::default::Default for $name {
            fn default() -> Self {
                Self::zeroed()
            }
        }

        // =============================================================================================================
        // Operators
        // =============================================================================================================

        crate::operator!(+ (commutative) |a: $name, b: $inner| -> $name { $name { m: [ $([ $( a[$c][$r] + b, )* ],)* ] } });
        crate::operator!(- (commutative) |a: $name, b: $inner| -> $name { $name { m: [ $([ $( a[$c][$r] - b, )* ],)* ] } });
        crate::operator!(* (commutative) |a: $name, b: $inner| -> $name { $name { m: [ $([ $( a[$c][$r] * b, )* ],)* ] } });
        crate::operator!(/ (commutative) |a: $name, b: $inner| -> $name { $name { m: [ $([ $( a[$c][$r] / b, )* ],)* ] } });
        crate::operator!(+= |a: &mut $name, b: $inner| { $($( a[$c][$r] += b;)*)* });
        crate::operator!(-= |a: &mut $name, b: $inner| { $($( a[$c][$r] -= b;)*)* });
        crate::operator!(*= |a: &mut $name, b: $inner| { $($( a[$c][$r] *= b;)*)* });
        crate::operator!(/= |a: &mut $name, b: $inner| { $($( a[$c][$r] /= b;)*)* });

        crate::operator!(+ |a: $name, b: $name| -> $name { $name { m: [ $([ $( a[$c][$r] + b[$c][$r], )* ],)* ] } });
        crate::operator!(- |a: $name, b: $name| -> $name { $name { m: [ $([ $( a[$c][$r] - b[$c][$r], )* ],)* ] } });
        crate::operator!(+= |a: &mut $name, b: $name| { $($( a[$c][$r] += b[$c][$r];)*)* });
        crate::operator!(-= |a: &mut $name, b: $name| { $($( a[$c][$r] -= b[$c][$r];)*)* });

        // =============================================================================================================
        // Core and mathematic implementations
        // =============================================================================================================

        impl $name {
            /// Creates a new matrix using column-major order.
            #[inline]
            pub const fn new_cm(
                $($( $param_n: $inner, )*)*
            ) -> Self {
                Self {
                    m: [$(
                        [ $( $param_n, )* ],
                    )*],
                }
            }

            /// Creates a new matrix using row-major order.
            #[inline]
            pub const fn new_rm(
                $($( $rm_param_src: $inner, )*)*
            ) -> Self {
                Self {
                    m: [$(
                        [ $( $rm_param_dst, )* ],
                    )*],
                }
            }

            /// Creates a new matrix from columns.
            ///
            /// Because vectors are identical to arrays, this operation should be a free move/copy when plain vectors
            /// are given.
            #[inline]
            pub fn from_cols< $($col_generic),* >( $($col_param: $col_generic),* ) -> Self
            where
                $($col_generic: Into<[$inner; $cols]>,)*
            {
                Self {
                    m: [
                        $($col_param.into(),)*
                    ]
                }
            }

            /// Creates a new matrix from rows.
            ///
            /// Because matrices are column-major, this operation cannot be optimized to a free copy of vectors; each element
            /// needs to be copied.
            pub fn from_rows< $($row_generic),* >( $($row_param: $row_generic),* ) -> Self
            where
                $($row_generic: Into<[$inner; $rows]>,)*
            {
                $( let $row_param: [$inner; $rows] = $row_param.into(); )*
                Self {
                    m: [$(
                        [ $( $fr_ident[$col_idx], )* ],
                    )*],
                }
            }

            /// Gets a pointer to the first entry of this matrix.
            ///
            /// Recall that matrices are stored in column-major order under the hood.
            pub const fn as_ptr(&self) -> *const $inner {
                // SAFETY: `repr(C)` guarantees that the address of `self` is the same as the first element.
                self as *const $name as *const $inner
            }

            /// Gets a mutable pointer to the first entry of this matrix.
            ///
            /// Recall that matrices are stored in column-major order under the hood.
            pub fn as_mut_ptr(&mut self) -> *mut $inner {
                // SAFETY: see `as_ptr`.
                self as *mut $name as *mut $inner
            }

            /// Gets a reference to this vector as a slice of bytes.
            pub fn as_bytes(&self) -> &[u8] {
                bytemuck::bytes_of(self)
            }

            /// Converts this vector to an array of bytes.
            pub fn to_bytes(self) -> [u8; ::core::mem::size_of::<$inner>() * $rows * $cols] {
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

        // Into 2D array
        // ----------------------------------------------------------

        impl ::core::convert::From<$name> for [[$inner; $rows]; $cols] {
            fn from(value: $name) -> Self {
                value.m
            }
        }

        impl ::core::convert::AsRef<[[$inner; $rows]; $cols]> for $name {
            fn as_ref(&self) -> &[[$inner; $rows]; $cols] {
                &self.m
            }
        }

        impl ::core::convert::AsMut<[[$inner; $rows]; $cols]> for $name {
            fn as_mut(&mut self) -> &mut [[$inner; $rows]; $cols] {
                &mut self.m
            }
        }

        impl ::core::borrow::Borrow<[[$inner; $rows]; $cols]> for $name {
            fn borrow(&self) -> &[[$inner; $rows]; $cols] {
                self.as_ref()
            }
        }

        impl ::core::borrow::BorrowMut<[[$inner; $rows]; $cols]> for $name {
            fn borrow_mut(&mut self) -> &mut [[$inner; $rows]; $cols] {
                self.as_mut()
            }
        }

        // Into array of column vectors
        // ----------------------------------------------------------

        impl ::core::convert::From<$name> for [$col_type; $cols] {
            fn from(value: $name) -> Self {
                value.m.map(|col| col.into())
            }
        }

        impl ::core::convert::AsRef<[$col_type; $cols]> for $name {
            fn as_ref(&self) -> &[$col_type; $cols] {
                // SAFETY: `repr(C)` on both matrix and vector types, coupled with Rust's array packing/alignment rules,
                // guarantee that (e.g.) a `Mat4` is equivalent to an array of four `Vec4`.
                unsafe { ::core::mem::transmute::<&$name, &[$col_type; $cols]>(self) }
            }
        }

        impl ::core::convert::AsMut<[$col_type; $cols]> for $name {
            fn as_mut(&mut self) -> &mut [$col_type; $cols] {
                // SAFETY: see `AsRef` impl above.
                unsafe { ::core::mem::transmute::<&mut $name, &mut [$col_type; $cols]>(self) }
            }
        }

        impl ::core::borrow::Borrow<[$col_type; $cols]> for $name {
            fn borrow(&self) -> &[$col_type; $cols] {
                self.as_ref()
            }
        }

        impl ::core::borrow::BorrowMut<[$col_type; $cols]> for $name {
            fn borrow_mut(&mut self) -> &mut [$col_type; $cols] {
                self.as_mut()
            }
        }

        // ----------------------------------------------------------

        impl $name {
            /// Gets a reference to this matrix, casted to a 2D array representation.
            ///
            /// This is equivalent to calling [`as_ref`][core::convert::AsRef::as_ref], but without the need for type
            /// annotations.
            #[inline]
            pub fn as_2d_array(&self) -> &[[$inner; $rows]; $cols] {
                self.as_ref()
            }

            /// Gets a reference to this matrix, casted to an array of column vectors.
            ///
            /// This is equivalent to calling [`as_ref`][core::convert::AsRef::as_ref], but without the need for type
            /// annotations.
            #[inline]
            pub fn as_columns(&self) -> &[$col_type; $cols] {
                self.as_ref()
            }

            /// Gets a mutable reference to this matrix, casted to a 2D array representation.
            ///
            /// This is equivalent to calling [`as_mut`][core::convert::AsMut::as_mut], but without the need for type
            /// annotations.
            #[inline]
            pub fn as_mut_2d_array(&mut self) -> &mut [[$inner; $rows]; $cols] {
                self.as_mut()
            }

            /// Gets a mutable reference to this matrix, casted to an array of column vectors.
            ///
            /// This is equivalent to calling [`as_mut`][core::convert::AsMut::as_mut], but without the need for type
            /// annotations.
            #[inline]
            pub fn as_mut_columns(&mut self) -> &mut [$col_type; $cols] {
                self.as_mut()
            }
        }

        // Conversions from inner type
        // ----------------------------------------------------------------------------------------

        // From 2D array
        // ----------------------------------------------------------

        impl ::core::convert::From<[[$inner; $rows]; $cols]> for $name {
            fn from(value: [[$inner; $rows]; $cols]) -> Self {
                $name { m: value }
            }
        }

        impl ::core::convert::TryFrom<&[[$inner; $rows]]> for $name {
            type Error = ::core::array::TryFromSliceError;

            fn try_from(value: &[[$inner; $rows]]) -> Result<Self, Self::Error> {
                let value: [[$inner; $rows]; $cols] = value.try_into()?;
                Ok(value.into())
            }
        }

        impl ::core::convert::TryFrom<&[&[$inner]]> for $name {
            type Error = ::core::array::TryFromSliceError;

            fn try_from(value: &[&[$inner]]) -> Result<Self, Self::Error> {
                // Copy slice of references to array of references
                let value: [&[$inner]; $cols] = value.try_into()?;
                // For each reference, copy to its own array in-line
                let value: [[$inner; $rows]; $cols] = [
                    $(value[$col_idx].try_into()?,)*
                ];
                Ok(value.into())
            }
        }

        impl ::core::convert::AsRef<$name> for [[$inner; $rows]; $cols] {
            fn as_ref(&self) -> &$name {
                // SAFETY: `repr(C)` coupled with Rust's packing/alignment rules for arrays (including nested ones)
                // guarantees that this array is the same as the struct and its single field.
                unsafe { ::core::mem::transmute::<&[[$inner; $rows]; $cols], &$name>(self) }
            }
        }

        impl ::core::convert::AsMut<$name> for [[$inner; $rows]; $cols] {
            fn as_mut(&mut self) -> &mut $name {
                // SAFETY: see `AsRef` impl above.
                unsafe { ::core::mem::transmute::<&mut [[$inner; $rows]; $cols], &mut $name>(self) }
            }
        }

        impl ::core::borrow::Borrow<$name> for [[$inner; $rows]; $cols] {
            fn borrow(&self) -> &$name {
                self.as_ref()
            }
        }

        impl ::core::borrow::BorrowMut<$name> for [[$inner; $rows]; $cols] {
            fn borrow_mut(&mut self) -> &mut $name {
                self.as_mut()
            }
        }

        // From array of column vectors
        // ----------------------------------------------------------

        impl ::core::convert::From<[$col_type; $cols]> for $name {
            fn from(value: [$col_type; $cols]) -> Self {
                // SAFETY: `repr(C)` on both matrix and vector types guarantees this.
                unsafe { ::core::mem::transmute::<[$col_type; $cols], $name>(value) }
            }
        }

        impl ::core::convert::TryFrom<&[$col_type]> for $name {
            type Error = ::core::array::TryFromSliceError;

            fn try_from(value: &[$col_type]) -> Result<Self, Self::Error> {
                let value: [$col_type; $cols] = value.try_into()?;
                Ok(value.into())
            }
        }

        impl ::core::convert::AsRef<$name> for [$col_type; $cols] {
            fn as_ref(&self) -> &$name {
                // SAFETY: see other `AsRef` impl.
                unsafe { ::core::mem::transmute::<&[$col_type; $cols], &$name>(self) }
            }
        }

        impl ::core::convert::AsMut<$name> for [$col_type; $cols] {
            fn as_mut(&mut self) -> &mut $name {
                // SAFETY: see other `AsMut` impl.
                unsafe { ::core::mem::transmute::<&mut [$col_type; $cols], &mut $name>(self) }
            }
        }

        impl ::core::borrow::Borrow<$name> for [$col_type; $cols] {
            fn borrow(&self) -> &$name {
                self.as_ref()
            }
        }

        impl ::core::borrow::BorrowMut<$name> for [$col_type; $cols] {
            fn borrow_mut(&mut self) -> &mut $name {
                self.as_mut()
            }
        }

        // Indexing
        // ----------------------------------------------------------------------------------------

        // Really what we want to do is to implement an indexer for everything that can index a slice. The nicest way to
        // do that would be blanket implementation over `I: SliceIndex<[Vec4]>`. However, if we do that, we can no
        // longer provide custom indexers for other types, like `[usize; 2]` or `(usize, usize)` (since, technically,
        // the Rust team could add implementations for `[usize; 2]: SliceIndex<[Vec4]>`, which would break our code).
        //
        // So, instead, we have to manually implement all the things that that blanket implementation would do. A list
        // of the types that implement the `SliceIndex` trait should start around here in the Rust docs:
        // https://doc.rust-lang.org/std/primitive.slice.html#impl-SliceIndex%3C%5BT%5D%3E-for-(Bound%3Cusize%3E,+Bound%3Cusize%3E)

        // These indexers return columns or ranges of columns:

        $crate::matrix::impl_matrix_basics!(@index $name, $col_type, usize);
        $crate::matrix::impl_matrix_basics!(@index $name, $col_type, ::core::ops::Range<usize>);
        $crate::matrix::impl_matrix_basics!(@index $name, $col_type, ::core::ops::RangeFrom<usize>);
        $crate::matrix::impl_matrix_basics!(@index $name, $col_type, ::core::ops::RangeFull);
        $crate::matrix::impl_matrix_basics!(@index $name, $col_type, ::core::ops::RangeInclusive<usize>);
        $crate::matrix::impl_matrix_basics!(@index $name, $col_type, ::core::ops::RangeTo<usize>);
        $crate::matrix::impl_matrix_basics!(@index $name, $col_type, ::core::ops::RangeToInclusive<usize>);
        $crate::matrix::impl_matrix_basics!(@index $name, $col_type, (::core::ops::Bound<usize>, ::core::ops::Bound<usize>));

        // These indexers use a 2-length array (`matrix[[4, 2]]`) to return an entry directly. Because `matrix[4][2]`
        // gets a column vector with the `4` and then indexes that with the `2`, that notation is column-major.
        // `matrix[[4, 2]]` uses row-major ordering.

        impl ::core::ops::Index<[usize; 2]> for $name {
            type Output = $inner;

            fn index(&self, index: [usize; 2]) -> &Self::Output {
                let row = index[0];
                let col = index[1];
                self.index(col).index(row)
            }
        }

        impl ::core::ops::IndexMut<[usize; 2]> for $name {
            fn index_mut(&mut self, index: [usize; 2]) -> &mut Self::Output {
                let row = index[0];
                let col = index[1];
                self.index_mut(col).index_mut(row)
            }
        }


        impl $name {
            /// Returns an iterator of reverences over all the entries in this matrix.
            pub fn entries(&self) -> impl Iterator<Item = &$inner> {
                self.as_2d_array().iter().flat_map(|arr| arr.iter())
            }

            /// Returns an iterator of mutable references over all the entries in this matrix.
            pub fn entries_mut(&mut self) -> impl Iterator<Item = &mut $inner> {
                self.as_mut_2d_array().iter_mut().flat_map(|arr| arr.iter_mut())
            }
        }
    };
    (@index $name:ident, $col_type:ty, $idx:ty) => {
        impl ::core::ops::Index<$idx> for $name {
            type Output = <$idx as core::slice::SliceIndex<[$col_type]>>::Output;

            fn index(&self, index: $idx) -> &Self::Output {
                self.as_columns().index(index)
            }
        }

        impl ::core::ops::IndexMut<$idx> for $name {
            fn index_mut(&mut self, index: $idx) -> &mut Self::Output {
                self.as_mut_columns().index_mut(index)
            }
        }
    };
}


use impl_matrix_basics;

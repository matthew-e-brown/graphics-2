/// Vectors and matrices; "base" data structures that [higher-level abstractions][mod@transforms] are built on top of.
mod base;

/// Higher-level abstractions built on top of [vectors and matrices][mod@base].
///
/// This includes:
///
/// - Points in 3D space, built on top of 3D vectors, but with a homogenous coordinate `w` of exactly `1`.
/// - Transformation matrices, built on top of 4D matrices, but with a fourth row of `[0 0 0 1]`.
mod transforms;

/// Models for geometric objects such as lines, planes, etc.
pub mod geometry;

pub use base::*;
pub use transforms::*;


/// Removes a leading or trailing `+` token from a repeated sequence.
macro_rules! strip_plus {
    (+ $($t:tt)*) => ($($t)*);
    ($($t:tt)* +) => ($($t)*);
}


/// Implements an operator for all reference/owned combinations.
///
/// Bodies are provided for reference operations, and then implementations for owned variants are generated from that.
macro_rules! operator {
    // Consumer-facing macro variants
    // ================================================================================

    // Regular binary operators
    ($op:tt $(#[$attr:meta])* |$lhs_name:ident: &$lhs_type:ty, $rhs_name:ident: &$rhs_type:ty$(,)?| -> $out_type:ty $body:block) => {
        $crate::operator!(@ $op, $(#[$attr])*, $lhs_name, $lhs_type, $rhs_name, $rhs_type, $out_type, $body);
    };

    // Assignment operators
    ($op:tt $(#[$attr:meta])* |$lhs_name:ident: &mut $lhs_type:ty, $rhs_name:ident: &$rhs_type:ty$(,)?| $body:block) => {
        $crate::operator!(@ $op, $(#[$attr])*, $lhs_name, $lhs_type, $rhs_name, $rhs_type, $body);
    };

    // Unary operators
    ($op:tt $(#[$attr:meta])* |$name:ident: &$type:ty$(,)?| -> $out_type:ty $body:block) => {
        $crate::operator!(@ $op (unary), $(#[$attr])*, $name, $type, $out_type, $body);
    };

    // Commutative binary operators (simply call the non-commutative version twice, once for each order)
    ($op:tt (commutative) $(#[$attr:meta])* |$lhs_name:ident: &$lhs_type:ty, $rhs_name:ident: &$rhs_type:ty$(,)?| -> $out_type:ty $body:block) => {
        $crate::operator!($op $(#[$attr])* |$lhs_name: &$lhs_type, $rhs_name: &$rhs_type| -> $out_type $body);
        $crate::operator!($op $(#[$attr])* |$rhs_name: &$rhs_type, $lhs_name: &$lhs_type| -> $out_type $body);
    };

    // Internal macro variants
    // ================================================================================

    // Parsing operator tokens to trait names
    // -------------------------------------------

    (@ +, $($rest:tt)*) => ($crate::operator!(@@ Add, add, $($rest)*););
    (@ -, $($rest:tt)*) => ($crate::operator!(@@ Sub, sub, $($rest)*););
    (@ *, $($rest:tt)*) => ($crate::operator!(@@ Mul, mul, $($rest)*););
    (@ /, $($rest:tt)*) => ($crate::operator!(@@ Div, div, $($rest)*););
    (@ %, $($rest:tt)*) => ($crate::operator!(@@ Rem, rem, $($rest)*););
    (@ &, $($rest:tt)*) => ($crate::operator!(@@ BitAnd, bit_and, $($rest)*););
    (@ |, $($rest:tt)*) => ($crate::operator!(@@ BitOr, bit_or, $($rest)*););
    (@ ^, $($rest:tt)*) => ($crate::operator!(@@ BitOr, bit_or, $($rest)*););
    (@ <<, $($rest:tt)*) => ($crate::operator!(@@ Shl, shl, $($rest)*););
    (@ >>, $($rest:tt)*) => ($crate::operator!(@@ Shr, shr, $($rest)*););

    (@ +=, $($rest:tt)*) => ($crate::operator!(@@ AddAssign, add_assign, $($rest)*););
    (@ -=, $($rest:tt)*) => ($crate::operator!(@@ SubAssign, sub_assign, $($rest)*););
    (@ *=, $($rest:tt)*) => ($crate::operator!(@@ MulAssign, mul_assign, $($rest)*););
    (@ /=, $($rest:tt)*) => ($crate::operator!(@@ DivAssign, div_assign, $($rest)*););
    (@ %=, $($rest:tt)*) => ($crate::operator!(@@ RemAssign, rem_assign, $($rest)*););
    (@ &=, $($rest:tt)*) => ($crate::operator!(@@ BitAndAssign, bit_and_assign, $($rest)*););
    (@ |=, $($rest:tt)*) => ($crate::operator!(@@ BitOrAssign, bit_or_assign, $($rest)*););
    (@ ^=, $($rest:tt)*) => ($crate::operator!(@@ BitOrAssign, bit_or_assign, $($rest)*););
    (@ <<=, $($rest:tt)*) => ($crate::operator!(@@ ShlAssign, shl_assign, $($rest)*););
    (@ >>=, $($rest:tt)*) => ($crate::operator!(@@ ShrAssign, shr_assign, $($rest)*););

    (@ - (unary), $($rest:tt)*) => ($crate::operator!(@@ Neg, neg, $($rest)*););
    (@ ! (unary), $($rest:tt)*) => ($crate::operator!(@@ Not, not, $($rest)*););

    // Trait implementation
    // -------------------------------------------

    // Binary operators
    (@@ $op_trait:ident, $op_fn:ident, $(#[$attr:meta])*, $lhs_name:ident, $lhs_type:ty, $rhs_name:ident, $rhs_type:ty, $out_type:ty, $body:block) => {
        // Implement base case with references
        impl ::core::ops::$op_trait<&$rhs_type> for &$lhs_type {
            type Output = $out_type;

            $(#[$attr])*
            fn $op_fn(self, $rhs_name: &$rhs_type) -> Self::Output {
                let $lhs_name = self;
                $body
            }
        }

        // Call that base case from the other combinations
        impl ::core::ops::$op_trait<&$rhs_type> for $lhs_type {
            type Output = $out_type;

            $(#[$attr])*
            fn $op_fn(self, rhs: &$rhs_type) -> Self::Output {
                (&self).$op_fn(rhs)
            }
        }

        impl ::core::ops::$op_trait<$rhs_type> for &$lhs_type {
            type Output = $out_type;

            $(#[$attr])*
            fn $op_fn(self, rhs: $rhs_type) -> Self::Output {
                self.$op_fn(&rhs)
            }
        }

        impl ::core::ops::$op_trait<$rhs_type> for $lhs_type {
            type Output = $out_type;

            $(#[$attr])*
            fn $op_fn(self, rhs: $rhs_type) -> Self::Output {
                (&self).$op_fn(&rhs)
            }
        }
    };

    // Assignment operators
    (@@ $op_trait:ident, $op_fn:ident, $(#[$attr:meta])*, $lhs_name:ident, $lhs_type:ty, $rhs_name:ident, $rhs_type:ty, $body:block) => {
        // Base case with reference
        impl ::core::ops::$op_trait<&$rhs_type> for $lhs_type {
            $(#[$attr])*
            fn $op_fn(&mut self, $rhs_name: &$rhs_type) {
                let $lhs_name = self;
                $body
            }
        }

        // Call base case from non-reference variant
        impl ::core::ops::$op_trait<$rhs_type> for $lhs_type {
            $(#[$attr])*
            fn $op_fn(&mut self, $rhs_name: $rhs_type) {
                self.$op_fn(&$rhs_name)
            }
        }
    };

    // Unary operators
    (@@ $op_trait:ident, $op_fn:ident, $(#[$attr:meta])*, $name:ident, $type:ty, $out_type:ty, $body:block) => {
        impl ::core::ops::$op_trait for &$type {
            type Output = $out_type;

            $(#[$attr])*
            fn $op_fn(self) -> Self::Output {
                let $name = self;
                $body
            }
        }

        impl ::core::ops::$op_trait for $type {
            type Output = $out_type;

            $(#[$attr])*
            fn $op_fn(self) -> Self::Output {
                (&self).$op_fn()
            }
        }
    };
}


pub(crate) use {operator, strip_plus};

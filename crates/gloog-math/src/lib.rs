pub mod matrix;
pub mod vector;


macro_rules! strip_plus {
    (+ $($t:tt)*) => ($($t)*);
}


/// Implements an operator for `A ⇄ B`, `&A ⇄ B`, `A ⇄ &B`, and `&A ⇄ &B` using the same body.
macro_rules! operator {
    // Consumer-facing macro variants
    // ================================================================================

    // Regular binary operators
    ($op:tt $(#[$attr:meta])* |$lhs_name:ident: $lhs_type:ty, $rhs_name:ident: $rhs_type:ty$(,)?| -> $out_type:ty $body:block) => {
        $crate::operator!(@ $op, $(#[$attr])*, $lhs_name, $lhs_type, $rhs_name, $rhs_type, $out_type, $body);
        $crate::operator!(@ $op, $(#[$attr])*, $lhs_name, &$lhs_type, $rhs_name, $rhs_type, $out_type, $body);
        $crate::operator!(@ $op, $(#[$attr])*, $lhs_name, $lhs_type, $rhs_name, &$rhs_type, $out_type, $body);
        $crate::operator!(@ $op, $(#[$attr])*, $lhs_name, &$lhs_type, $rhs_name, &$rhs_type, $out_type, $body);
    };

    // Assignment operators
    ($op:tt $(#[$attr:meta])* |$lhs_name:ident: &mut $lhs_type:ty, $rhs_name:ident: $rhs_type:ty$(,)?| $body:block) => {
        $crate::operator!(@ $op, $(#[$attr])*, $lhs_name, $lhs_type, $rhs_name, $rhs_type, $body);
        $crate::operator!(@ $op, $(#[$attr])*, $lhs_name, $lhs_type, $rhs_name, &$rhs_type, $body);
    };

    // Unary operators
    ($op:tt $(#[$attr:meta])* |$name:ident: $type:ty$(,)?| -> $out_type:ty $body:block) => {
        $crate::operator!(@ $op, $(#[$attr])*, name, $type, $out_type, $body);
    };

    // Commutative binary operators (simply call the non-commutative version twice)
    ($op:tt (commutative) $(#[$attr:meta])* |$lhs_name:ident: $lhs_type:ty, $rhs_name:ident: $rhs_type:ty$(,)?| -> $out_type:ty $body:block) => {
        $crate::operator!($op $(#[$attr])* |$lhs_name:$lhs_type, $rhs_name:$rhs_type| -> $out_type $body);
        $crate::operator!($op $(#[$attr])* |$rhs_name:$rhs_type, $lhs_name:$lhs_type| -> $out_type $body);
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

    (@ -, $($rest:tt)*) => ($crate::operator!(@@ Neg, neg, $($rest)*););
    (@ !, $($rest:tt)*) => ($crate::operator!(@@ Not, not, $($rest)*););

    // Trait implementation
    // -------------------------------------------

    // Binary operators
    (@@ $op_trait:ident, $op_fn:ident, $(#[$attr:meta])*, $lhs_name:ident, $lhs_type:ty, $rhs_name:ident, $rhs_type:ty, $out_type:ty, $body:block) => {
        impl ::core::ops::$op_trait<$rhs_type> for $lhs_type {
            type Output = $out_type;

            $(#[$attr])*
            fn $op_fn(self, $rhs_name: $rhs_type) -> Self::Output {
                let $lhs_name = self;
                $body
            }
        }
    };

    // Assignment operators
    (@@ $op_trait:ident, $op_fn:ident, $(#[$attr:meta])*, $lhs_name:ident, $lhs_type:ty, $rhs_name:ident, $rhs_type:ty, $body:block) => {
        impl ::core::ops::$op_trait<$rhs_type> for $lhs_type {
            $(#[$attr])*
            fn $op_fn(&mut self, $rhs_name: $rhs_type) {
                let $lhs_name = self;
                $body
            }
        }
    };

    // Unary operators
    (@@ $op_trait:ident, $op_fn:ident, $(#[$attr:meta])*, $name:ident, $type:ty, $out_type:ty, $body:block) => {
        impl ::core::ops::$op_trait for $type {
            type Output = $out_type;
            $(#[$attr])*
            fn $op_fn(self) -> Self::Output {
                let $name = self;
                $body
            }
        }
    };
}


pub(crate) use {operator, strip_plus};

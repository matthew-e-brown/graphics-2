//! Matrices.
//!
//! See [the crate-level documentation](self) for general details that pertain to all data structures in the crate.

math_proc::create_matrix!(pub struct Mat2, f32, 2, 2);
math_proc::create_matrix!(pub struct Mat3, f32, 3, 3);
math_proc::create_matrix!(pub struct Mat4, f32, 4, 4);

math_proc::matrix_impl_scalar_ops!(Mat2, f32, 2, 2);
math_proc::matrix_impl_scalar_ops!(Mat3, f32, 3, 3);
math_proc::matrix_impl_scalar_ops!(Mat4, f32, 4, 4);

math_proc::matrix_impl_self_ops!(Mat2, f32, 2, 2);
math_proc::matrix_impl_self_ops!(Mat3, f32, 3, 3);
math_proc::matrix_impl_self_ops!(Mat4, f32, 4, 4);

math_proc::matrix_impl_row_col_conversions!(Mat2, f32, 2, 2);
math_proc::matrix_impl_row_col_conversions!(Mat3, f32, 3, 3);
math_proc::matrix_impl_row_col_conversions!(Mat4, f32, 4, 4);


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[rustfmt::skip]
    fn constructors() {
        let a2 = Mat2::new(
            1.0, 2.0,
            3.0, 4.0,
        );
        let b2 = Mat2 { m: [
            [1.0, 3.0],
            [2.0, 4.0],
        ] };

        let a3 = Mat3::new(
            1.0, 2.0, 3.0,
            4.0, 5.0, 6.0,
            7.0, 8.0, 9.0,
        );
        let b3 = Mat3 { m: [
            [1.0, 4.0, 7.0],
            [2.0, 5.0, 8.0],
            [3.0, 6.0, 9.0],
        ] };

        let a4 = Mat4::new(
             1.0,  2.0,  3.0,  4.0,
             5.0,  6.0,  7.0,  8.0,
             9.0, 10.0, 11.0, 12.0,
            13.0, 14.0, 15.0, 16.0,
        );
        let b4 = Mat4 { m: [
            [1.0, 5.0,  9.0, 13.0],
            [2.0, 6.0, 10.0, 14.0],
            [3.0, 7.0, 11.0, 15.0],
            [4.0, 8.0, 12.0, 16.0],
        ] };

        assert_eq!(a2.m, b2.m);
        assert_eq!(a3.m, b3.m);
        assert_eq!(a4.m, b4.m);
    }
}

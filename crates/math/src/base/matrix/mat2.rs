use bytemuck::{Pod, Zeroable};

use crate::Vector2D;


/// A 2×2 matrix of 32-bit floats.
///
/// This struct is `repr(C)`, so it is guaranteed to be identical to `[[f32; 2]; 2]` or `[f32; 4]`.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Pod, Zeroable)]
pub struct Matrix2D {
    m: [[f32; 2]; 2],
}


#[rustfmt::skip]
super::impl_matrix_basics!(Matrix2D, f32, 2 * 2 (16 bytes), {
    col_type: Vector2D,
    col_order: [
        c0/C0/0: [n00: (0, 0), n10: (0, 1)] / [r0, r1],
        c1/C1/1: [n01: (1, 0), n11: (1, 1)] / [r0, r1],
    ],
    fr_params: [r0/R0, r1/R1],
    rm_mapping: [
        [n00 -> n00, n01 -> n10],
        [n10 -> n01, n11 -> n11],
    ],
});


#[rustfmt::skip]
crate::operator!(* |a: &Matrix2D, b: &Matrix2D| -> Matrix2D {
    Matrix2D::new(
        /* row 0 -------------------------------------------------- */
            /* col 0 */ (a[[0,0]] * b[[0,0]]) + (a[[0,1]] * b[[1,0]]),
            /* col 1 */ (a[[0,0]] * b[[0,1]]) + (a[[0,1]] * b[[1,1]]),
        /* row 1 -------------------------------------------------- */
            /* col 0 */ (a[[1,0]] * b[[0,0]]) + (a[[1,1]] * b[[1,0]]),
            /* col 1 */ (a[[1,0]] * b[[0,1]]) + (a[[1,1]] * b[[1,1]]),
    )
});

#[rustfmt::skip]
crate::operator!(* |a: &Matrix2D, b: &Vector2D| -> Vector2D {
    Vector2D::new(
        a[0][0] * b.x   +   a[1][0] * b.y,
        a[0][1] * b.x   +   a[1][1] * b.y,
    )
});


impl Matrix2D {
    /// The 2×2 identity matrix.
    #[rustfmt::skip]
    pub const IDENTITY: Matrix2D = Matrix2D::new(
        1.0, 0.0,
        0.0, 1.0,
    );

    /// Computes a new matrix which is this matrix's transpose.
    #[inline]
    #[rustfmt::skip]
    pub fn transpose(&self) -> Matrix2D {
        Matrix2D::new(
            self[[0, 0]], self[[1, 0]],
            self[[0, 1]], self[[1, 1]],
        )
    }

    /// Computes the determinant of this matrix.
    #[inline]
    pub fn det(&self) -> f32 {
        self[[0, 0]] * self[[1, 1]] - self[[0, 1]] * self[[1, 0]]
    }

    /// Computes this matrix's inverse.
    ///
    /// In the interest of performance, there is no check for whether or not this matrix is invertible (if its
    /// determinant of zero).
    #[rustfmt::skip]
    pub fn inverse(&self) -> Matrix2D {
        let inv_det = 1.0 / self.det();
        let inv_neg = -inv_det;

        Matrix2D::new(
            inv_det * self[[1, 1]], inv_neg * self[[0, 1]],
            inv_neg * self[[1, 0]], inv_det * self[[0, 0]],
        )
    }
}

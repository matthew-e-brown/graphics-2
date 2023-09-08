use bytemuck::{Pod, Zeroable};

use crate::Vector3D;

/// A 3×3 matrix of 32-bit floats.
///
/// This struct is `repr(C)`, so it is guaranteed to be identical to `[[f32; 3]; 3]` or `[f32; 9]`.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Pod, Zeroable)]
pub struct Matrix3D {
    m: [[f32; 3]; 3],
}


#[rustfmt::skip]
super::impl_matrix_basics!(Matrix3D, f32, 3 * 3 (36 bytes), {
    col_type: Vector3D,
    col_order: [
        c0/C0/0: [n00: (0, 0), n10: (0, 1), n20: (0, 2)] / [r0, r1, r2],
        c1/C1/1: [n01: (1, 0), n11: (1, 1), n21: (1, 2)] / [r0, r1, r2],
        c2/C2/2: [n02: (2, 0), n12: (2, 1), n22: (2, 2)] / [r0, r1, r2],
    ],
    fr_params: [r0/R0, r1/R1, r2/R2],
    rm_mapping: [
        [n00 -> n00, n01 -> n10, n02 -> n20],
        [n10 -> n01, n11 -> n11, n12 -> n21],
        [n20 -> n02, n21 -> n12, n22 -> n22],
    ],
});


#[rustfmt::skip]
crate::operator!(* |a: &Matrix3D, b: &Matrix3D| -> Matrix3D {
    Matrix3D::new_cm(
        a[0][0] * b[0][0]   +   a[1][0] * b[0][1]   +   a[2][0] * b[0][2], // row 1, col 1
        a[0][1] * b[0][0]   +   a[1][1] * b[0][1]   +   a[2][1] * b[0][2], // row 2, col 1
        a[0][2] * b[0][0]   +   a[1][2] * b[0][1]   +   a[2][2] * b[0][2], // row 3, col 1
        // ---------------------------------------------------------------
        a[0][0] * b[1][0]   +   a[1][0] * b[1][1]   +   a[2][0] * b[1][2], // row 1, col 2
        a[0][1] * b[1][0]   +   a[1][1] * b[1][1]   +   a[2][1] * b[1][2], // row 2, col 2
        a[0][2] * b[1][0]   +   a[1][2] * b[1][1]   +   a[2][2] * b[1][2], // row 3, col 2
        // ---------------------------------------------------------------
        a[0][0] * b[2][0]   +   a[1][0] * b[2][1]   +   a[2][0] * b[2][2], // row 1, col 3
        a[0][1] * b[2][0]   +   a[1][1] * b[2][1]   +   a[2][1] * b[2][2], // row 2, col 3
        a[0][2] * b[2][0]   +   a[1][2] * b[2][1]   +   a[2][2] * b[2][2], // row 3, col 3
    )
});

#[rustfmt::skip]
crate::operator!(* |a: &Matrix3D, b: &Vector3D| -> Vector3D {
    Vector3D::new(
        a[0][0] * b.x   +   a[1][0] * b.y   +   a[2][0] * b.z,
        a[0][1] * b.x   +   a[1][1] * b.y   +   a[2][1] * b.z,
        a[0][2] * b.x   +   a[1][2] * b.y   +   a[2][2] * b.z,
    )
});


impl Matrix3D {
    /// The 3×3 identity matrix.
    #[rustfmt::skip]
    pub const IDENTITY: Matrix3D = Matrix3D::new(
        1.0, 0.0, 0.0,
        0.0, 1.0, 0.0,
        0.0, 0.0, 1.0,
    );

    /// Computes a new matrix which is this matrix's transpose.
    #[rustfmt::skip]
    pub fn transpose(&self) -> Matrix3D {
        Matrix3D::new(
            self[[0, 0]], self[[1, 0]], self[[2, 0]],
            self[[0, 1]], self[[1, 1]], self[[2, 1]],
            self[[0, 2]], self[[1, 2]], self[[2, 2]],
        )
    }

    /// Computes the determinant of this matrix.
    pub fn det(&self) -> f32 {
        // See equation 1.94 and 1.95 (p. 47/48) [Foundations of Game Development, Vol. 1]
        Vector3D::scalar_triple(&self[0], &self[1], &self[2])
    }

    /// Computes this matrix's inverse.
    ///
    /// In the interest of performance, there is no check for whether or not this matrix is invertible (if its
    /// determinant of zero).
    #[rustfmt::skip]
    pub fn inverse(&self) -> Matrix3D {
        let a = &self[0];
        let b = &self[1];
        let c = &self[2];

        let r0 = b.cross(c);
        let r1 = c.cross(a);
        let r2 = a.cross(b);

        let inv_det = 1.0 / r2.dot(c);

        Matrix3D::new(
            r0.x * inv_det, r0.y * inv_det, r0.z * inv_det,
            r1.x * inv_det, r1.y * inv_det, r1.z * inv_det,
            r2.x * inv_det, r2.y * inv_det, r2.z * inv_det,
        )
    }
}
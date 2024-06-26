use bytemuck::{Pod, Zeroable};

use crate::{Mat4, Vec3};

/// A 3×3 matrix of 32-bit floats.
///
/// This struct is `repr(C)`, so it is guaranteed to be identical to `[[f32; 3]; 3]` or `[f32; 9]`.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Pod, Zeroable)]
pub struct Mat3 {
    m: [[f32; 3]; 3],
}


#[rustfmt::skip]
super::impl_matrix_basics!(Mat3, f32, 3 * 3 (36 bytes), {
    col_type: Vec3,
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
crate::operator!(* |a: &Mat3, b: &Mat3| -> Mat3 {
    Mat3::new(
        /* row 0 -------------------------------------------------------------------------- */
            /* col 0 */ (a[[0,0]] * b[[0,0]]) + (a[[0,1]] * b[[1,0]]) + (a[[0,2]] * b[[2,0]]),
            /* col 1 */ (a[[0,0]] * b[[0,1]]) + (a[[0,1]] * b[[1,1]]) + (a[[0,2]] * b[[2,1]]),
            /* col 2 */ (a[[0,0]] * b[[0,2]]) + (a[[0,1]] * b[[1,2]]) + (a[[0,2]] * b[[2,2]]),
        /* row 1 -------------------------------------------------------------------------- */
            /* col 0 */ (a[[1,0]] * b[[0,0]]) + (a[[1,1]] * b[[1,0]]) + (a[[1,2]] * b[[2,0]]),
            /* col 1 */ (a[[1,0]] * b[[0,1]]) + (a[[1,1]] * b[[1,1]]) + (a[[1,2]] * b[[2,1]]),
            /* col 2 */ (a[[1,0]] * b[[0,2]]) + (a[[1,1]] * b[[1,2]]) + (a[[1,2]] * b[[2,2]]),
        /* row 2 -------------------------------------------------------------------------- */
            /* col 0 */ (a[[2,0]] * b[[0,0]]) + (a[[2,1]] * b[[1,0]]) + (a[[2,2]] * b[[2,0]]),
            /* col 1 */ (a[[2,0]] * b[[0,1]]) + (a[[2,1]] * b[[1,1]]) + (a[[2,2]] * b[[2,1]]),
            /* col 2 */ (a[[2,0]] * b[[0,2]]) + (a[[2,1]] * b[[1,2]]) + (a[[2,2]] * b[[2,2]]),
    )
});

#[rustfmt::skip]
crate::operator!(* |a: &Mat3, b: &Vec3| -> Vec3 {
    Vec3::new(
        a[0][0] * b.x   +   a[1][0] * b.y   +   a[2][0] * b.z,
        a[0][1] * b.x   +   a[1][1] * b.y   +   a[2][1] * b.z,
        a[0][2] * b.x   +   a[1][2] * b.y   +   a[2][2] * b.z,
    )
});


impl Mat3 {
    /// The 3×3 identity matrix.
    #[rustfmt::skip]
    pub const IDENTITY: Mat3 = Mat3::new(
        1.0, 0.0, 0.0,
        0.0, 1.0, 0.0,
        0.0, 0.0, 1.0,
    );

    /// Computes a new matrix which is this matrix's transpose.
    #[rustfmt::skip]
    pub fn transpose(&self) -> Mat3 {
        Mat3::new(
            self[[0, 0]], self[[1, 0]], self[[2, 0]],
            self[[0, 1]], self[[1, 1]], self[[2, 1]],
            self[[0, 2]], self[[1, 2]], self[[2, 2]],
        )
    }

    /// Creates a [`Mat3`] from a [`Mat4`] by trimming out the last row and column.
    #[inline]
    #[rustfmt::skip]
    pub fn from_mat4(mat: &Mat4) -> Mat3 {
        Mat3::new(
            mat[[0,0]], mat[[0,1]], mat[[0,2]],
            mat[[1,0]], mat[[1,1]], mat[[1,2]],
            mat[[2,0]], mat[[2,1]], mat[[2,2]],
        )
    }

    /// Computes the determinant of this matrix.
    pub fn det(&self) -> f32 {
        // See equation 1.94 and 1.95 (p. 47/48) [Foundations of Game Development, Vol. 1]
        Vec3::scalar_triple(&self[0], &self[1], &self[2])
    }

    /// Computes this matrix's inverse.
    ///
    /// In the interest of performance, there is no check for whether or not this matrix is invertible (if its
    /// determinant of zero).
    #[rustfmt::skip]
    pub fn inverse(&self) -> Mat3 {
        let a = &self[0];
        let b = &self[1];
        let c = &self[2];

        let r0 = b.cross(c);
        let r1 = c.cross(a);
        let r2 = a.cross(b);

        let inv_det = 1.0 / r2.dot(c);

        Mat3::new(
            r0.x * inv_det, r0.y * inv_det, r0.z * inv_det,
            r1.x * inv_det, r1.y * inv_det, r1.z * inv_det,
            r2.x * inv_det, r2.y * inv_det, r2.z * inv_det,
        )
    }
}

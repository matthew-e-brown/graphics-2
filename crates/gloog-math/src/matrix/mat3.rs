use bytemuck::{Pod, Zeroable};

use crate::vector::Vec3;


#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Pod, Zeroable)]
pub struct Mat3 {
    m: [[f32; 3]; 3],
}

#[rustfmt::skip]
super::impl_matrix_basics!(Mat3, f32, 3 * 3, {
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
crate::operator!(* |a: Mat3, b: Mat3| -> Mat3 {
    Mat3::new(
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
crate::operator!(* |a: Mat3, b: Vec3| -> Vec3 {
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
            self[0][0], self[1][0], self[2][0],
            self[0][1], self[1][1], self[2][1],
            self[0][2], self[1][2], self[2][2],
        )
    }
}

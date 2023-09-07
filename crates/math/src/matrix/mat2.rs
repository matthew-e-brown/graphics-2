use bytemuck::{Pod, Zeroable};

use crate::vector::Vec2;


#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Pod, Zeroable)]
pub struct Mat2 {
    m: [[f32; 2]; 2],
}

#[rustfmt::skip]
super::impl_matrix_basics!(Mat2, f32, 2 * 2, {
    col_type: Vec2,
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
crate::operator!(* |a: Mat2, b: Mat2| -> Mat2 {
    Mat2::new_cm(
        a[0][0] * b[0][0]   +   a[1][0] * b[0][1], // row 1, col 1
        a[0][1] * b[0][0]   +   a[1][1] * b[0][1], // row 2, col 1
        // ---------------------------------------
        a[0][0] * b[1][0]   +   a[1][0] * b[1][1], // row 1, col 2
        a[0][1] * b[1][0]   +   a[1][1] * b[1][1], // row 2, col 2
    )
});

#[rustfmt::skip]
crate::operator!(* |a: Mat2, b: Vec2| -> Vec2 {
    Vec2::new(
        a[0][0] * b.x   +   a[1][0] * b.y,
        a[0][1] * b.x   +   a[1][1] * b.y,
    )
});

impl Mat2 {
    /// The 2×2 identity matrix.
    #[rustfmt::skip]
    pub const IDENTITY: Mat2 = Mat2::new_cm(
        1.0, 0.0,
        0.0, 1.0,
    );

    /// Computes a new matrix which is this matrix's transpose.
    #[rustfmt::skip]
    pub fn transpose(&self) -> Mat2 {
        Mat2::new_rm(
            self[[0, 0]], self[[1, 0]],
            self[[0, 1]], self[[1, 1]],
        )
    }
}

use bytemuck::{Pod, Zeroable};

use crate::Vector4D;


/// A 4×4 matrix of 32-bit floats.
///
/// This struct is `repr(C)`, so it is guaranteed to be identical to `[[f32; 4]; 4]` or `[f32; 16]`.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Pod, Zeroable)]
pub struct Matrix4D {
    m: [[f32; 4]; 4],
}


#[rustfmt::skip]
super::impl_matrix_basics!(Matrix4D, f32, 4 * 4 (64 bytes), {
    col_type: Vector4D,
    col_order: [
        c0/C0/0: [n00: (0, 0), n10: (0, 1), n20: (0, 2), n30: (0, 3)] / [r0, r1, r2, r3],
        c1/C1/1: [n01: (1, 0), n11: (1, 1), n21: (1, 2), n31: (1, 3)] / [r0, r1, r2, r3],
        c2/C2/2: [n02: (2, 0), n12: (2, 1), n22: (2, 2), n32: (2, 3)] / [r0, r1, r2, r3],
        c3/C3/3: [n03: (3, 0), n13: (3, 1), n23: (3, 2), n33: (3, 3)] / [r0, r1, r2, r3],
    ],
    fr_params: [r0/R0, r1/R1, r2/R2, r3/R3],
    rm_mapping: [
        [n00 -> n00, n01 -> n10, n02 -> n20, n03 -> n30],
        [n10 -> n01, n11 -> n11, n12 -> n21, n13 -> n31],
        [n20 -> n02, n21 -> n12, n22 -> n22, n23 -> n32],
        [n30 -> n03, n31 -> n13, n32 -> n23, n33 -> n33],
    ],
});


#[rustfmt::skip]
crate::operator!(* |a: &Matrix4D, b: &Matrix4D| -> Matrix4D {
    Matrix4D::new_cm(
        a[0][0] * b[0][0]   +   a[1][0] * b[0][1]   +   a[2][0] * b[0][2]   +   a[3][0] * b[0][3], // row 1, col 1
        a[0][0] * b[1][0]   +   a[1][0] * b[1][1]   +   a[2][0] * b[1][2]   +   a[3][0] * b[1][3], // row 2, col 1
        a[0][0] * b[2][0]   +   a[1][0] * b[2][1]   +   a[2][0] * b[2][2]   +   a[3][0] * b[2][3], // row 3, col 1
        a[0][0] * b[3][0]   +   a[1][0] * b[3][1]   +   a[2][0] * b[3][2]   +   a[3][0] * b[3][3], // row 4, col 1
        // ---------------------------------------------------------------------------------------
        a[0][1] * b[0][0]   +   a[1][1] * b[0][1]   +   a[2][1] * b[0][2]   +   a[3][1] * b[0][3], // row 1, col 2
        a[0][1] * b[1][0]   +   a[1][1] * b[1][1]   +   a[2][1] * b[1][2]   +   a[3][1] * b[1][3], // row 2, col 2
        a[0][1] * b[2][0]   +   a[1][1] * b[2][1]   +   a[2][1] * b[2][2]   +   a[3][1] * b[2][3], // row 3, col 2
        a[0][1] * b[3][0]   +   a[1][1] * b[3][1]   +   a[2][1] * b[3][2]   +   a[3][1] * b[3][3], // row 4, col 2
        // ---------------------------------------------------------------------------------------
        a[0][2] * b[0][0]   +   a[1][2] * b[0][1]   +   a[2][2] * b[0][2]   +   a[3][2] * b[0][3], // row 1, col 3
        a[0][2] * b[1][0]   +   a[1][2] * b[1][1]   +   a[2][2] * b[1][2]   +   a[3][2] * b[1][3], // row 2, col 3
        a[0][2] * b[2][0]   +   a[1][2] * b[2][1]   +   a[2][2] * b[2][2]   +   a[3][2] * b[2][3], // row 3, col 3
        a[0][2] * b[3][0]   +   a[1][2] * b[3][1]   +   a[2][2] * b[3][2]   +   a[3][2] * b[3][3], // row 4, col 3
        // ---------------------------------------------------------------------------------------
        a[0][3] * b[0][0]   +   a[1][3] * b[0][1]   +   a[2][3] * b[0][2]   +   a[3][3] * b[0][3], // row 1, col 4
        a[0][3] * b[1][0]   +   a[1][3] * b[1][1]   +   a[2][3] * b[1][2]   +   a[3][3] * b[1][3], // row 2, col 4
        a[0][3] * b[2][0]   +   a[1][3] * b[2][1]   +   a[2][3] * b[2][2]   +   a[3][3] * b[2][3], // row 3, col 4
        a[0][3] * b[3][0]   +   a[1][3] * b[3][1]   +   a[2][3] * b[3][2]   +   a[3][3] * b[3][3], // row 4, col 4
    )
});

#[rustfmt::skip]
crate::operator!(* |a: &Matrix4D, b: &Vector4D| -> Vector4D {
    Vector4D::new(
        a[0][0] * b.x   +   a[1][0] * b.y   +   a[2][0] * b.z   +   a[3][0] * b.w,
        a[0][1] * b.x   +   a[1][1] * b.y   +   a[2][1] * b.z   +   a[3][1] * b.w,
        a[0][2] * b.x   +   a[1][2] * b.y   +   a[2][2] * b.z   +   a[3][2] * b.w,
        a[0][3] * b.x   +   a[1][3] * b.y   +   a[2][3] * b.z   +   a[3][3] * b.w,
    )
});


impl Matrix4D {
    /// The 4×4 identity matrix.
    #[rustfmt::skip]
    pub const IDENTITY: Matrix4D = Matrix4D::new(
        1.0, 0.0, 0.0, 0.0,
        0.0, 1.0, 0.0, 0.0,
        0.0, 0.0, 1.0, 0.0,
        0.0, 0.0, 0.0, 1.0,
    );

    /// Computes a new matrix which is this matrix's transpose.
    #[rustfmt::skip]
    pub fn transpose(&self) -> Matrix4D {
        Matrix4D::new(
            self[[0, 0]], self[[1, 0]], self[[2, 0]], self[[3, 0]],
            self[[0, 1]], self[[1, 1]], self[[2, 1]], self[[3, 1]],
            self[[0, 2]], self[[1, 2]], self[[2, 2]], self[[3, 2]],
            self[[0, 3]], self[[1, 3]], self[[2, 3]], self[[3, 3]],
        )
    }
}

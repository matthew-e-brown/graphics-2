use bytemuck::{Pod, Zeroable};

use crate::{Mat3, Vec3, Vec4};


/// A 4×4 matrix of 32-bit floats.
///
/// This struct is `repr(C)`, so it is guaranteed to be identical to `[[f32; 4]; 4]` or `[f32; 16]`.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Pod, Zeroable)]
pub struct Mat4 {
    m: [[f32; 4]; 4],
}


#[rustfmt::skip]
super::impl_matrix_basics!(Mat4, f32, 4 * 4 (64 bytes), {
    col_type: Vec4,
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
crate::operator!(* |a: &Mat4, b: &Mat4| -> Mat4 {
    Mat4::new(
        /* row 0 -------------------------------------------------------------------------------------------------- */
            /* col 0 */ (a[[0,0]] * b[[0,0]]) + (a[[0,1]] * b[[1,0]]) + (a[[0,2]] * b[[2,0]]) + (a[[0,3]] * b[[3,0]]),
            /* col 1 */ (a[[0,0]] * b[[0,1]]) + (a[[0,1]] * b[[1,1]]) + (a[[0,2]] * b[[2,1]]) + (a[[0,3]] * b[[3,1]]),
            /* col 2 */ (a[[0,0]] * b[[0,2]]) + (a[[0,1]] * b[[1,2]]) + (a[[0,2]] * b[[2,2]]) + (a[[0,3]] * b[[3,2]]),
            /* col 3 */ (a[[0,0]] * b[[0,3]]) + (a[[0,1]] * b[[1,3]]) + (a[[0,2]] * b[[2,3]]) + (a[[0,3]] * b[[3,3]]),
        /* row 1 -------------------------------------------------------------------------------------------------- */
            /* col 0 */ (a[[1,0]] * b[[0,0]]) + (a[[1,1]] * b[[1,0]]) + (a[[1,2]] * b[[2,0]]) + (a[[1,3]] * b[[3,0]]),
            /* col 1 */ (a[[1,0]] * b[[0,1]]) + (a[[1,1]] * b[[1,1]]) + (a[[1,2]] * b[[2,1]]) + (a[[1,3]] * b[[3,1]]),
            /* col 2 */ (a[[1,0]] * b[[0,2]]) + (a[[1,1]] * b[[1,2]]) + (a[[1,2]] * b[[2,2]]) + (a[[1,3]] * b[[3,2]]),
            /* col 3 */ (a[[1,0]] * b[[0,3]]) + (a[[1,1]] * b[[1,3]]) + (a[[1,2]] * b[[2,3]]) + (a[[1,3]] * b[[3,3]]),
        /* row 2 -------------------------------------------------------------------------------------------------- */
            /* col 0 */ (a[[2,0]] * b[[0,0]]) + (a[[2,1]] * b[[1,0]]) + (a[[2,2]] * b[[2,0]]) + (a[[2,3]] * b[[3,0]]),
            /* col 1 */ (a[[2,0]] * b[[0,1]]) + (a[[2,1]] * b[[1,1]]) + (a[[2,2]] * b[[2,1]]) + (a[[2,3]] * b[[3,1]]),
            /* col 2 */ (a[[2,0]] * b[[0,2]]) + (a[[2,1]] * b[[1,2]]) + (a[[2,2]] * b[[2,2]]) + (a[[2,3]] * b[[3,2]]),
            /* col 3 */ (a[[2,0]] * b[[0,3]]) + (a[[2,1]] * b[[1,3]]) + (a[[2,2]] * b[[2,3]]) + (a[[2,3]] * b[[3,3]]),
        /* row 3 -------------------------------------------------------------------------------------------------- */
            /* col 0 */ (a[[3,0]] * b[[0,0]]) + (a[[3,1]] * b[[1,0]]) + (a[[3,2]] * b[[2,0]]) + (a[[3,3]] * b[[3,0]]),
            /* col 1 */ (a[[3,0]] * b[[0,1]]) + (a[[3,1]] * b[[1,1]]) + (a[[3,2]] * b[[2,1]]) + (a[[3,3]] * b[[3,1]]),
            /* col 2 */ (a[[3,0]] * b[[0,2]]) + (a[[3,1]] * b[[1,2]]) + (a[[3,2]] * b[[2,2]]) + (a[[3,3]] * b[[3,2]]),
            /* col 3 */ (a[[3,0]] * b[[0,3]]) + (a[[3,1]] * b[[1,3]]) + (a[[3,2]] * b[[2,3]]) + (a[[3,3]] * b[[3,3]]),
    )
});

#[rustfmt::skip]
crate::operator!(* |a: &Mat4, b: &Vec4| -> Vec4 {
    Vec4::new(
        a[0][0] * b.x   +   a[1][0] * b.y   +   a[2][0] * b.z   +   a[3][0] * b.w,
        a[0][1] * b.x   +   a[1][1] * b.y   +   a[2][1] * b.z   +   a[3][1] * b.w,
        a[0][2] * b.x   +   a[1][2] * b.y   +   a[2][2] * b.z   +   a[3][2] * b.w,
        a[0][3] * b.x   +   a[1][3] * b.y   +   a[2][3] * b.z   +   a[3][3] * b.w,
    )
});


impl Mat4 {
    /// The 4×4 identity matrix.
    #[rustfmt::skip]
    pub const IDENTITY: Mat4 = Mat4::new(
        1.0, 0.0, 0.0, 0.0,
        0.0, 1.0, 0.0, 0.0,
        0.0, 0.0, 1.0, 0.0,
        0.0, 0.0, 0.0, 1.0,
    );

    /// Computes a new matrix which is this matrix's transpose.
    #[rustfmt::skip]
    pub fn transpose(&self) -> Mat4 {
        Mat4::new(
            self[[0, 0]], self[[1, 0]], self[[2, 0]], self[[3, 0]],
            self[[0, 1]], self[[1, 1]], self[[2, 1]], self[[3, 1]],
            self[[0, 2]], self[[1, 2]], self[[2, 2]], self[[3, 2]],
            self[[0, 3]], self[[1, 3]], self[[2, 3]], self[[3, 3]],
        )
    }

    /// Creates a [`Mat3`] by trimming out the last row and column of this matrix.
    #[inline]
    #[rustfmt::skip]
    pub fn to_mat3(&self) -> Mat3 {
        Mat3::new(
            self[[0,0]], self[[0,1]], self[[0,2]],
            self[[1,0]], self[[1,1]], self[[1,2]],
            self[[2,0]], self[[2,1]], self[[2,2]],
        )
    }

    /// Function for accessing the columns of this 4D matrix as 3D vectors, getting the bottom row directly, and
    /// calculating the intermediate vectors `s`, `t`, `u`, and `v`.
    ///
    /// These values are all used for computing both the matrix determinant and inverse. Because both functions use the
    /// same first few calculations, they are written once and (hopefully) inlined into both methods. [`det`] probably
    /// won't see a whole lot of use, but the other matrices have `det` functions, so we may as well provide a `det`
    /// function here as well.
    ///
    /// See p.47-50 in Foundations of Game Dev, vol.1 for information on what this is all about.
    #[inline(always)]
    fn inv_det_helper(&self) -> ([&Vec3; 4], [&f32; 4], [Vec3; 4]) {
        // SAFETY: this is the same sort of cast + re-borrow we do for `as_array` or `as_columns`. The only reason we
        // don't use `as_columns` or simply index this matrix is because we want our vec4 columns treated as vec3.
        let a: &Vec3 = unsafe { &*self[0].as_ptr().cast() };
        let b: &Vec3 = unsafe { &*self[1].as_ptr().cast() };
        let c: &Vec3 = unsafe { &*self[2].as_ptr().cast() };
        let d: &Vec3 = unsafe { &*self[3].as_ptr().cast() };

        let x = &self[[3, 0]];
        let y = &self[[3, 1]];
        let z = &self[[3, 2]];
        let w = &self[[3, 3]];

        let s = a.cross(b);
        let t = c.cross(d);
        let u = (y * a) - (x * b);
        let v = (w * c) - (z * d);

        ([a, b, c, d], [x, y, z, w], [s, t, u, v])
    }

    /// Computes the determinant of this matrix.
    pub fn det(&self) -> f32 {
        let (_, _, [s, t, u, v]) = self.inv_det_helper();
        s.dot(&v) + t.dot(&u)
    }

    /// Computes this matrix's inverse.
    ///
    /// In the interest of performance, there is no check for whether or not this matrix is invertible (if its
    /// determinant of zero).
    pub fn inverse(&self) -> Mat4 {
        let ([a, b, c, d], [x, y, z, w], [mut s, mut t, mut u, mut v]) = self.inv_det_helper();

        let inv_det = 1.0 / (s.dot(&v) + t.dot(&u));
        s *= inv_det;
        t *= inv_det;
        u *= inv_det;
        v *= inv_det;

        let r0 = b.cross(&v) + (y * t);
        let r1 = v.cross(&a) - (x * t);
        let r2 = d.cross(&u) + (w * s);
        let r3 = u.cross(&c) - (z * s);

        #[rustfmt::skip]
        return Mat4::new(
            r0.x, r0.y, r0.z, -b.dot(&t),
            r1.x, r1.y, r1.z,  a.dot(&t),
            r2.x, r2.y, r2.z, -d.dot(&s),
            r3.x, r3.y, r3.z,  c.dot(&s),
        );

        // explicit return: https://github.com/rust-lang/rust/issues/15701
    }
}

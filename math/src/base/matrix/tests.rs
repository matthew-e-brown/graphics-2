fn cmp_f32(a: f32, b: f32) -> bool {
    let abs_diff = (a - b).abs();
    abs_diff <= 0.001
}

fn cmp_entries<const R: usize, const C: usize>(expect: &[[f32; R]; C], actual: &[[f32; R]; C]) {
    for r in 0..R {
        for c in 0..C {
            assert!(
                cmp_f32(expect[c][r], actual[c][r]),
                "expect[{r}, {c}] != actual[{r}, {c}] --- {:.8} != {:.8}",
                expect[c][r],
                actual[c][r]
            );
        }
    }
}

mod mat2 {
    use super::super::Matrix2D;
    use super::cmp_entries;

    #[test]
    fn mul() {
        #[rustfmt::skip]
        let a = Matrix2D::new(
            5.1, 8.9,
            -3.6, 7.5,
        );

        #[rustfmt::skip]
        let b = Matrix2D::new(
            -1.2, -4.2,
            6.0, 0.0
        );

        #[rustfmt::skip]
        let expect = Matrix2D::new(
            47.28, -21.42,
            49.32, 15.12,
        );

        let actual = a * b;

        cmp_entries(expect.as_2d_array(), actual.as_2d_array());
    }
}

mod mat3 {
    use super::super::Matrix3D;
    use super::cmp_entries;

    #[test]
    fn mul() {
        #[rustfmt::skip]
        let a = Matrix3D::new(
            -18.1, 14.0, 0.0,
            9.2, 1.0, 1.0,
            2.0, 8.4, 32.0,
        );

        #[rustfmt::skip]
        let b = Matrix3D::new(
            8.0, 7.0, 6.0,
            -12.0, 1.0, 0.0,
            2.7, 0.0, -7.6,
        );

        #[rustfmt::skip]
        let expect = Matrix3D::new(
            -312.8, -112.7, -108.6,
            64.3, 65.4, 47.6,
            1.6, 22.4, -231.2,
        );

        let actual = a * b;

        cmp_entries(expect.as_2d_array(), actual.as_2d_array());
    }
}

mod mat4 {
    use super::super::Matrix4D;
    use super::cmp_entries;

    #[test]
    fn mul1() {
        #[rustfmt::skip]
        let a = Matrix4D::new(
            -5.5, 7.0, 9.1, 10.0,
            2.0, 3.9, -3.4, 8.2,
            8.2, 10.3, 2.0, -3.5,
            3.1, -3.6, 4.4, 8.2,
        );

        #[rustfmt::skip]
        let b = Matrix4D::new(
            3.9, 10.1, -12.6, 18.7,
            -12.6, 1.2, 4.2, 9.9,
            9.0, 10.0, 12.7, -2.4,
            3.1, -12.3, 4.9, 10.5,
        );

        #[rustfmt::skip]
        let expect = Matrix4D::new(
            3.25, -79.15, 263.27, 49.61,
            -46.52, -109.98, -11.82, 170.27,
            -90.65, 158.23, -51.81, 213.76,
            122.47, -29.87, 41.88, 97.87,
        );

        let actual = a * b;

        cmp_entries(expect.as_2d_array(), actual.as_2d_array());
    }

    #[test]
    fn mul2() {
        #[rustfmt::skip]
        let a = Matrix4D::new(
            5.0, 7.0, 9.0, 10.0,
            2.0, 3.0, 3.0, 8.0,
            8.0, 10.0, 2.0, 3.0,
            3.0, 3.0, 4.0, 8.0,
        );

        #[rustfmt::skip]
        let b = Matrix4D::new(
            3.0, 10.0, 12.0, 18.0,
            12.0, 1.0, 4.0, 9.0,
            9.0, 10.0, 12.0, 2.0,
            3.0, 12.0, 4.0, 10.0,
        );

        #[rustfmt::skip]
        let expect = Matrix4D::new(
            210.0, 267.0, 236.0, 271.0,
            93.0, 149.0, 104.0, 149.0,
            171.0, 146.0, 172.0, 268.0,
            105.0, 169.0, 128.0, 169.0,
        );

        let actual = a * b;

        cmp_entries(expect.as_2d_array(), actual.as_2d_array());
    }
}

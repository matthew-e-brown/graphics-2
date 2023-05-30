//! Vectors.
//!
//! See [the crate-level documentation](self) for general details that pertain to all data structures in the crate.


math_proc::create_vector! {
    #[derive(Copy, Clone, Debug, PartialEq)]
    pub struct Vec2;
    f32, 2;
}

math_proc::create_vector! {
    #[derive(Copy, Clone, Debug, PartialEq)]
    pub struct Vec3;
    f32, 3;
}

math_proc::create_vector! {
    #[derive(Copy, Clone, Debug, PartialEq)]
    pub struct Vec4;
    f32, 4;
}


math_proc::vector_impl_scalar_ops!(Vec2, f32, 2);
math_proc::vector_impl_scalar_ops!(Vec3, f32, 3);
math_proc::vector_impl_scalar_ops!(Vec4, f32, 4);

math_proc::vector_impl_self_ops!(Vec2, f32, 2);
math_proc::vector_impl_self_ops!(Vec3, f32, 3);
math_proc::vector_impl_self_ops!(Vec4, f32, 4);


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constructors() {
        let v2 = Vec2::new(1.0, 2.0);
        let w2 = Vec2 { v: [1.0, 2.0] };

        let v3 = Vec3::new(1.0, 2.0, 3.0);
        let w3 = Vec3 { v: [1.0, 2.0, 3.0] };

        let v4 = Vec4::new(1.0, 2.0, 3.0, 4.0);
        let w4 = Vec4 { v: [1.0, 2.0, 3.0, 4.0] };

        assert_eq!(v2.v, w2.v);
        assert_eq!(v3.v, w3.v);
        assert_eq!(v4.v, w4.v);
    }

    #[test]
    fn into_array() {
        let vec = Vec2::new(1.0, 2.0);
        let arr: [f32; 2] = vec.into();
        assert_eq!(vec.v, arr);

        let vec = Vec3::new(3.0, 4.0, 5.0);
        let arr = <[f32; 3]>::from(vec);
        assert_eq!(vec.v, arr);

        let vec = Vec4::new(6.0, 7.0, 8.0, 9.0);
        let arr: [f32; 4] = vec.into();
        assert_eq!(vec.v, arr);
    }


    #[test]
    fn scalar_ops() {
        let x = Vec3::from([ 1.0, 2.0, 3.0 ]);
        assert_eq!((x * 10.0).v, Vec3::new(10.0, 20.0, 30.0).v);
        assert_eq!((x / 2.0).v, Vec3::new(0.5, 1.0, 1.5).v);
    }
}

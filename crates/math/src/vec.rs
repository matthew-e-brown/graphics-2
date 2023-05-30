//! Vectors.
//!
//! See [the crate-level documentation](self) for general details that pertain to all data structures in the crate.

math_proc::create_vector!(pub struct Vec2, f32, 2);
math_proc::create_vector!(pub struct Vec3, f32, 3);
math_proc::create_vector!(pub struct Vec4, f32, 4);

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
        let a = Vec2::new(1.0, 2.0);
        let b = Vec2 { v: [1.0, 2.0] };

        assert_eq!(a.v, b.v);
        assert_eq!(b.v, [1.0, 2.0]);

        let a = Vec3::new(1.0, 2.0, 3.0);
        let b = Vec3 { v: [1.0, 2.0, 3.0] };

        assert_eq!(a.v, b.v);
        assert_eq!(b.v, [1.0, 2.0, 3.0]);

        let a = Vec4::new(1.0, 2.0, 3.0, 4.0);
        let b = Vec4 { v: [1.0, 2.0, 3.0, 4.0] };

        assert_eq!(a.v, b.v);
        assert_eq!(b.v, [1.0, 2.0, 3.0, 4.0]);
    }
}

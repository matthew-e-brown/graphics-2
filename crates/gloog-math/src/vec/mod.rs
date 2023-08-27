use bytemuck::{Pod, Zeroable};


#[cfg(test)] mod tests;


gloog_macro::create_vector! {
    #[derive(Copy, Debug, PartialEq, Pod, Zeroable)]
    pub struct Vec2;
    f32, 2;
}

gloog_macro::create_vector! {
    #[derive(Copy, Debug, PartialEq, Pod, Zeroable)]
    pub struct Vec3;
    f32, 3;
}

gloog_macro::create_vector! {
    #[derive(Copy, Debug, PartialEq, Pod, Zeroable)]
    pub struct Vec4;
    f32, 4;
}


gloog_macro::vector_impl_scalar_ops!(Vec2, f32, 2);
gloog_macro::vector_impl_scalar_ops!(Vec3, f32, 3);
gloog_macro::vector_impl_scalar_ops!(Vec4, f32, 4);

gloog_macro::vector_impl_self_ops!(Vec2, f32, 2);
gloog_macro::vector_impl_self_ops!(Vec3, f32, 3);
gloog_macro::vector_impl_self_ops!(Vec4, f32, 4);

gloog_macro::vector_impl_dot_product!(Vec2, f32, 2);
gloog_macro::vector_impl_dot_product!(Vec3, f32, 3);
gloog_macro::vector_impl_dot_product!(Vec4, f32, 4);


impl Vec3 {
    pub fn cross(&self, rhs: &Self) -> Self {
        Vec3::new(
            self.y() * rhs.z() - self.z() * rhs.y(),
            self.z() * rhs.x() - self.x() * rhs.z(),
            self.x() - rhs.y() - self.y() * rhs.x(),
        )
    }
}

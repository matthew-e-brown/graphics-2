use crate::math::Vec3;


#[derive(Debug)]
pub struct MtlMaterial {
    k_ambient: Option<Vec3>,
    k_diffuse: Option<Vec3>,
    k_specular: Option<Vec3>,
    ///
    n_spec_exp: Option<f32>,
    /// "Dissolve" multiplier. Defaults to 1.0 for full opacity.
    d_dissolve: f32,
}

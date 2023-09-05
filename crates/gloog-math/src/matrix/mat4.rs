use core::default::Default;

use bytemuck::{Pod, Zeroable};


#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Pod, Zeroable)]
pub struct Mat4 {
    m: [[f32; 4]; 4],
}

impl Default for Mat4 {
    fn default() -> Self {
        Self::zeroed()
    }
}

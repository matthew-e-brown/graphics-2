use core::default::Default;

use bytemuck::{Pod, Zeroable};


#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Pod, Zeroable)]
pub struct Mat2 {
    m: [[f32; 2]; 2],
}

impl Default for Mat2 {
    fn default() -> Self {
        Self::zeroed()
    }
}

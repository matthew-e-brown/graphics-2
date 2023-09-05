use core::default::Default;

use bytemuck::{Pod, Zeroable};


#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Pod, Zeroable)]
pub struct Mat3 {
    m: [[f32; 3]; 3],
}

impl Default for Mat3 {
    fn default() -> Self {
        Self::zeroed()
    }
}

mod buffers;
mod shaders;
mod uniforms;
mod vertex;

use crate::types::{ClearMask, EnableCap};
use crate::GLContext;


impl GLContext {
    pub fn viewport(&self, x: i32, y: i32, width: i32, height: i32) {
        unsafe { self.funcs.viewport(x, y, width, height) }
    }

    pub fn clear_color(&self, red: f32, green: f32, blue: f32, alpha: f32) {
        unsafe { self.funcs.clear_color(red, green, blue, alpha) }
    }

    pub fn enable(&self, cap: EnableCap) {
        unsafe { self.funcs.enable(cap.into()) }
    }

    pub fn clear(&self, mask: ClearMask) {
        unsafe { self.funcs.clear(mask.into()) }
    }
}

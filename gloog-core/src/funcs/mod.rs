mod buffers;
mod shaders;
mod uniforms;
mod vertex;

use crate::types::{ClearMask, EnableCap, PolygonMode, PolygonModeFace};
use crate::GLContext;


impl GLContext {
    pub fn viewport(&self, x: i32, y: i32, width: i32, height: i32) {
        unsafe { self.gl.viewport(x, y, width, height) }
    }


    pub fn clear_color(&self, red: f32, green: f32, blue: f32, alpha: f32) {
        unsafe { self.gl.clear_color(red, green, blue, alpha) }
    }


    pub fn enable(&self, cap: EnableCap) {
        unsafe { self.gl.enable(cap.into_raw()) }
    }


    pub fn clear(&self, mask: ClearMask) {
        unsafe { self.gl.clear(mask.into_raw()) }
    }


    pub fn polygon_mode(&self, face: PolygonModeFace, mode: PolygonMode) {
        unsafe { self.gl.polygon_mode(face.into_raw(), mode.into_raw()) }
    }
}

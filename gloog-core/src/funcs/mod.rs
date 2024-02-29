mod buffers;
mod debug;
mod shaders;
mod uniforms;
mod vertex;

use std::ffi::CStr;

use crate::types::*;
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

    pub fn disable(&self, cap: EnableCap) {
        unsafe { self.gl.disable(cap.into_raw()) }
    }

    pub fn clear(&self, mask: ClearMask) {
        unsafe { self.gl.clear(mask.into_raw()) }
    }

    pub fn polygon_mode(&self, face: PolygonModeFace, mode: PolygonMode) {
        unsafe { self.gl.polygon_mode(face.into_raw(), mode.into_raw()) }
    }

    pub fn cull_face(&self, mode: TriangleFace) {
        unsafe { self.gl.cull_face(mode.into_raw()) }
    }

    pub fn front_face(&self, mode: FrontFaceDirection) {
        unsafe { self.gl.front_face(mode.into_raw()) }
    }

    pub fn get_string(&self, name: StringName) -> String {
        let ptr = unsafe { self.gl.get_string(name.into_raw()) };
        // SAFETY: I think it's probably reasonable to trust strings from OpenGL...?
        let str = unsafe { CStr::from_ptr(ptr.cast()) };
        str.to_string_lossy().into_owned()
    }

    pub fn get_string_i(&self, name: IndexedStringName, index: u32) -> String {
        let ptr = unsafe { self.gl.get_string_i(name.into_raw(), index) };
        // SAFETY: I think it's probably reasonable to trust strings from OpenGL...?
        let str = unsafe { CStr::from_ptr(ptr.cast()) };
        str.to_string_lossy().into_owned()
    }
}

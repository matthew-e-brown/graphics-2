pub(crate) mod buffers;
pub(crate) mod debug;
pub(crate) mod shaders;
pub(crate) mod uniforms;
pub(crate) mod vertex;

use std::ffi::CStr;

use crate::raw::types::*;
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

    pub fn polygon_mode(&self, mode: PolygonMode) {
        /// From `glPolygonMode` (p.480/502, section section 14.6.4) in the 4.6 core spec, `face` must be
        /// `FRONT_AND_BACK`.
        const FACE: GLenum = PolygonFace::FrontAndBack.into_raw();
        unsafe { self.gl.polygon_mode(FACE, mode.into_raw()) }
    }

    pub fn cull_face(&self, mode: PolygonFace) {
        unsafe { self.gl.cull_face(mode.into_raw()) }
    }

    pub fn front_face(&self, mode: FrontFaceDirection) {
        unsafe { self.gl.front_face(mode.into_raw()) }
    }

    pub fn get_string(&self, name: StringName) -> String {
        let ptr = unsafe { self.gl.get_string(name.into_raw()) };
        // SAFETY: OpenGL 4.6 core spec (p.566/588, section 22.2) says that "string queries return pointers to UTF-8
        // encoded, null-terminated static strings describing properties of the current GL context." So this pointer
        // should always be valid, unless an implementation provides a bogus one.
        let str = unsafe { CStr::from_ptr(ptr.cast()) };
        str.to_string_lossy().into_owned()
    }

    pub fn get_string_i(&self, name: IndexedStringName, index: u32) -> String {
        let ptr = unsafe { self.gl.get_string_i(name.into_raw(), index) };
        // SAFETY: See `get_string`.
        let str = unsafe { CStr::from_ptr(ptr.cast()) };
        str.to_string_lossy().into_owned()
    }
}

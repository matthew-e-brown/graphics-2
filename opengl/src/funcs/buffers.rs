use crate::bindings::types::*;
use crate::types::*;
use crate::{convert, GLContext};


impl GLContext {
    pub fn create_buffer(&self) -> BufferID {
        let mut name = 0;
        unsafe { self.gl.create_buffers(1, &mut name) };
        BufferID::new(name)
    }


    pub fn create_buffers(&self, n: usize) -> Vec<BufferID> {
        if n == 0 {
            return vec![];
        }

        let mut names = vec![0; n];
        let n = convert!(n, GLsizei, "number of buffers");

        unsafe { self.gl.create_buffers(n, names.as_mut_ptr()) };
        names.into_iter().map(BufferID::new).collect()
    }


    pub fn delete_buffer(&self, buffer: BufferID) {
        unsafe { self.gl.delete_buffers(1, &buffer.into_raw()) }
    }


    pub fn delete_buffers(&self, buffers: &[BufferID]) {
        let len = convert!(buffers.len(), GLsizei, "number of buffers");
        let ptr = buffers.as_ptr().cast(); // cast is safe because `BufferID` is `repr(transparent)`
        unsafe { self.gl.delete_buffers(len, ptr) }
    }


    pub fn bind_buffer(&self, target: BufferTarget, buffer: BufferID) {
        unsafe { self.gl.bind_buffer(target.into_raw(), buffer.into_raw()) }
    }


    pub fn unbind_buffer(&self, target: BufferTarget) {
        unsafe { self.gl.bind_buffer(target.into_raw(), 0) }
    }


    pub fn buffer_data(&self, target: BufferTarget, data: &[u8], usage: BufferUsage) {
        let len = convert!(data.len(), isize, "buffer data size");
        let ptr = data.as_ptr().cast();
        unsafe { self.gl.buffer_data(target.into_raw(), len, ptr, usage.into_raw()) }
    }


    pub fn named_buffer_data(&self, buffer: BufferID, data: &[u8], usage: BufferUsage) {
        let len = convert!(data.len(), isize, "buffer data size");
        let ptr = data.as_ptr().cast();
        unsafe { self.gl.named_buffer_data(buffer.into_raw(), len, ptr, usage.into_raw()) }
    }
}

use crate::types::*;
use crate::{convert, GLContext};

impl GLContext {
    pub fn create_buffer(&self) -> BufferID {
        let mut name = 0;
        unsafe { self.funcs.create_buffers(1, &mut name) };
        BufferID::new(name)
    }

    pub fn create_buffers(&self, n: usize) -> Vec<BufferID> {
        if n == 0 {
            return vec![];
        }

        let mut names = vec![0; n];
        let n = convert!(n, i32, "number of buffers");

        unsafe { self.funcs.create_buffers(n, names.as_mut_ptr()) };
        names.into_iter().map(BufferID::new).collect()
    }

    pub fn delete_buffer(&self, buffer: BufferID) {
        // cast is safe because `BufferID` is `repr(transparent)`
        unsafe { self.funcs.delete_buffers(1, (&buffer) as *const BufferID as *const u32) }
    }

    pub fn delete_buffers(&self, buffers: &[BufferID]) {
        let len = convert!(buffers.len(), i32, "number of buffers");
        let ptr = buffers.as_ptr().cast(); // cast is safe because `BufferID` is `repr(transparent)`
        unsafe { self.funcs.delete_buffers(len, ptr) }
    }

    pub fn bind_buffer(&self, target: BufferTarget, buffer: BufferID) {
        unsafe { self.funcs.bind_buffer(target.into(), buffer.into()) }
    }

    pub fn unbind_buffer(&self, target: BufferTarget) {
        unsafe { self.funcs.bind_buffer(target.into(), 0) }
    }

    pub fn buffer_data(&self, target: BufferTarget, data: &[u8], usage: BufferUsage) {
        let len = convert!(data.len(), isize, "buffer data size");
        let ptr = data.as_ptr().cast();
        unsafe { self.funcs.buffer_data(target.into(), len, ptr, usage.into()) }
    }

    pub fn named_buffer_data(&self, buffer: BufferID, data: &[u8], usage: BufferUsage) {
        let len = convert!(data.len(), isize, "buffer data size");
        let ptr = data.as_ptr().cast();
        unsafe { self.funcs.named_buffer_data(buffer.into(), len, ptr, usage.into()) }
    }
}

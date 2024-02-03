use gl::types::*;

use crate::gl_convert;
use crate::types::*;


pub fn create_buffer() -> BufferID {
    let mut name = 0;
    unsafe {
        gl::CreateBuffers(1, &mut name);
    }

    BufferID::new(name)
}


pub fn create_buffers(n: usize) -> Vec<BufferID> {
    if n == 0 {
        return vec![];
    }

    let mut names = vec![0; n];
    let n = gl_convert!(n, GLsizei, "number of buffers");

    unsafe {
        gl::CreateBuffers(n, names.as_mut_ptr());
    }

    names.into_iter().map(BufferID::new).collect()
}


pub fn delete_buffer(buffer: BufferID) {
    unsafe {
        // cast is safe because `BufferID` is `repr(transparent)`
        gl::DeleteBuffers(1, &buffer as *const BufferID as *const _)
    }
}


pub fn delete_buffers(buffers: &[BufferID]) {
    let n = gl_convert!(buffers.len(), GLsizei, "number of buffers");
    let p = buffers.as_ptr().cast(); // cast is safe because `BufferID` is `repr(transparent)`
    unsafe {
        gl::DeleteBuffers(n, p);
    }
}


pub fn bind_buffer(target: BufferTarget, buffer: BufferID) {
    unsafe {
        gl::BindBuffer(target.raw(), buffer.raw());
    }
}


pub fn buffer_data(target: BufferTarget, data: &[u8], usage: BufferUsage) {
    let size = gl_convert!(data.len(), GLsizeiptr, "buffer data size");
    unsafe {
        gl::BufferData(target.raw(), size, data.as_ptr().cast(), usage.raw());
    }
}


pub fn named_buffer_data(buffer: BufferID, data: &[u8], usage: BufferUsage) {
    let size = gl_convert!(data.len(), GLsizeiptr, "buffer data size");
    unsafe {
        gl::NamedBufferData(buffer.raw(), size, data.as_ptr().cast(), usage.raw());
    }
}

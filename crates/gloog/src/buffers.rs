use gl::types::*;

use crate::types::*;


pub fn create_buffer() -> Buffer {
    let mut name = 0;
    unsafe {
        gl::CreateBuffers(1, &mut name);
    }

    Buffer::new(name)
}


pub fn create_buffers(n: usize) -> Vec<Buffer> {
    if n == 0 {
        return vec![];
    }

    let mut names = vec![0; n];
    let n: GLsizei = n.try_into().expect("buffer creation count should fit into `GLsizei`");

    unsafe {
        gl::CreateBuffers(n, names.as_mut_ptr());
    }

    names.into_iter().map(|name| Buffer::new(name)).collect()
}


pub fn bind_buffer(target: BufferTarget, buffer: &Buffer) {
    unsafe {
        gl::BindBuffer(target.into(), buffer.name());
    }
}


pub fn buffer_data(target: BufferTarget, data: impl AsRef<[u8]>, usage: BufferUsage) {
    let data = data.as_ref();
    let size: GLsizeiptr = data.len().try_into().expect("buffer data size should fit into `GLsizeiptr`");
    unsafe {
        gl::BufferData(target.into(), size, data.as_ptr().cast(), usage.into());
    }
}


pub fn named_buffer_data(buffer: &Buffer, data: impl AsRef<[u8]>, usage: BufferUsage) {
    let data = data.as_ref();
    let size: GLsizeiptr = data.len().try_into().expect("buffer data size should fit into `GLsizeiptr`");
    unsafe {
        gl::NamedBufferData(buffer.name(), size, data.as_ptr().cast(), usage.into());
    }
}

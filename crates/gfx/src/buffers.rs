use std::fmt::Display;

use gl::types::*;

use crate::RawData;


/// Acceptable values for [buffer][Buffer] binding targets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BufferTarget {
    /// Buffer target for vertex attributes.
    ArrayBuffer,
    /// Buffer target for atomic counter storage.
    AtomicCounterBuffer,
    /// Buffer target for the source of buffer copies.
    CopyReadBuffer,
    /// Buffer target for the destination of buffer copies.
    CopyWriteBuffer,
    /// Buffer target for indirect compute dispatch commands.
    DispatchIndirectBuffer,
    /// Buffer target for indirect command arguments.
    DrawIndirectBuffer,
    /// Buffer target for vertex array indices.
    ElementArrayBuffer,
    /// Buffer target for the destination of pixel read operations.
    PixelPackBuffer,
    /// Buffer target for the source of texture data.
    PixelUnpackBuffer,
    /// Buffer target for the query results.
    QueryBuffer,
    /// Buffer target for read-write storage for shaders.
    ShaderStorageBuffer,
    /// Buffer target for texture data.
    TextureBuffer,
    /// Buffer target for transform feedback data.
    TransformFeedbackBuffer,
    /// Buffer target for uniform block storage.
    UniformBuffer,
}

impl From<BufferTarget> for GLenum {
    fn from(value: BufferTarget) -> Self {
        match value {
            BufferTarget::ArrayBuffer => gl::ARRAY_BUFFER,
            BufferTarget::AtomicCounterBuffer => gl::ATOMIC_COUNTER_BUFFER,
            BufferTarget::CopyReadBuffer => gl::COPY_READ_BUFFER,
            BufferTarget::CopyWriteBuffer => gl::COPY_WRITE_BUFFER,
            BufferTarget::DispatchIndirectBuffer => gl::DISPATCH_INDIRECT_BUFFER,
            BufferTarget::DrawIndirectBuffer => gl::DRAW_INDIRECT_BUFFER,
            BufferTarget::ElementArrayBuffer => gl::ELEMENT_ARRAY_BUFFER,
            BufferTarget::PixelPackBuffer => gl::PIXEL_PACK_BUFFER,
            BufferTarget::PixelUnpackBuffer => gl::PIXEL_UNPACK_BUFFER,
            BufferTarget::QueryBuffer => gl::QUERY_BUFFER,
            BufferTarget::ShaderStorageBuffer => gl::SHADER_STORAGE_BUFFER,
            BufferTarget::TextureBuffer => gl::TEXTURE_BUFFER,
            BufferTarget::TransformFeedbackBuffer => gl::TRANSFORM_FEEDBACK_BUFFER,
            BufferTarget::UniformBuffer => gl::UNIFORM_BUFFER,
        }
    }
}

impl Display for BufferTarget {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str(match self {
            BufferTarget::ArrayBuffer => "GL_ARRAY_BUFFER",
            BufferTarget::AtomicCounterBuffer => "GL_ATOMIC_COUNTER_BUFFER",
            BufferTarget::CopyReadBuffer => "GL_COPY_READ_BUFFER",
            BufferTarget::CopyWriteBuffer => "GL_COPY_WRITE_BUFFER",
            BufferTarget::DispatchIndirectBuffer => "GL_DISPATCH_INDIRECT_BUFFER",
            BufferTarget::DrawIndirectBuffer => "GL_DRAW_INDIRECT_BUFFER",
            BufferTarget::ElementArrayBuffer => "GL_ELEMENT_ARRAY_BUFFER",
            BufferTarget::PixelPackBuffer => "GL_PIXEL_PACK_BUFFER",
            BufferTarget::PixelUnpackBuffer => "GL_PIXEL_UNPACK_BUFFER",
            BufferTarget::QueryBuffer => "GL_QUERY_BUFFER",
            BufferTarget::ShaderStorageBuffer => "GL_SHADER_STORAGE_BUFFER",
            BufferTarget::TextureBuffer => "GL_TEXTURE_BUFFER",
            BufferTarget::TransformFeedbackBuffer => "GL_TRANSFORM_FEEDBACK_BUFFER",
            BufferTarget::UniformBuffer => "GL_UNIFORM_BUFFER",
        })
    }
}

/// Acceptable usage patterns for [buffer][Buffer] data stores.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DataUsage {
    /// The data store contents will be modified once and used at most a few times; the contents are modified by the
    /// application, and used as the source for GL drawing and image specification commands.
    StreamDraw,
    /// The data store contents will be modified once and used at most a few times; the contents are modified by reading
    /// data from the GL, and used to return that data when queried by the application.
    StreamRead,
    /// The data store contents will be modified once and used at most a few times; the contents are modified by reading
    /// data from the GL, and used as the source for GL drawing and image specification commands.
    StreamCopy,
    /// The data store contents will be modified once and used many times; the contents are modified by the application,
    /// and used as the source for GL drawing and image specification commands.
    StaticDraw,
    /// The data store contents will be modified once and used many times; the contents are modified by reading data
    /// from the GL, and used to return that data when queried by the application.
    StaticRead,
    /// The data store contents will be modified once and used many times; the contents are modified by reading data
    /// from the GL, and used as the source for GL drawing and image specification commands.
    StaticCopy,
    /// The data store contents will be modified repeatedly and used many times; the contents are modified by the
    /// application, and used as the source for GL drawing and image specification commands.
    DynamicDraw,
    /// The data store contents will be modified repeatedly and used many times; the contents are modified by reading
    /// data from the GL, and used to return that data when queried by the application.
    DynamicRead,
    /// The data store contents will be modified repeatedly and used many times; the contents are modified by reading
    /// data from the GL, and used as the source for GL drawing and image specification commands.
    DynamicCopy,
}

impl From<DataUsage> for GLenum {
    fn from(value: DataUsage) -> Self {
        match value {
            DataUsage::StreamDraw => gl::STREAM_DRAW,
            DataUsage::StreamRead => gl::STREAM_READ,
            DataUsage::StreamCopy => gl::STREAM_COPY,
            DataUsage::StaticDraw => gl::STATIC_DRAW,
            DataUsage::StaticRead => gl::STATIC_READ,
            DataUsage::StaticCopy => gl::STATIC_COPY,
            DataUsage::DynamicDraw => gl::DYNAMIC_DRAW,
            DataUsage::DynamicRead => gl::DYNAMIC_READ,
            DataUsage::DynamicCopy => gl::DYNAMIC_COPY,
        }
    }
}


/// An arbitrary buffer in OpenGL.
#[derive(Debug)]
pub struct Buffer {
    name: GLuint,
}

impl Drop for Buffer {
    fn drop(&mut self) {
        unsafe { gl::DeleteBuffers(1, &self.name) }
    }
}

impl Buffer {
    /// The _"name"_ that OpenGL uses for this buffer under the hood.
    pub fn gl_name(&self) -> GLuint {
        self.name
    }

    /// Creates a new buffer object.
    pub fn new() -> Self {
        let mut name = 0;

        unsafe {
            gl::CreateBuffers(1, &mut name);
        }

        Self { name }
    }

    /// Creates one or more buffer objects.
    ///
    /// # Panics
    ///
    /// * If `n` is zero.
    pub fn new_multiple(n: usize) -> Vec<Self> {
        assert!(n > 0, "cannot create zero buffer objects");

        let mut names = vec![0; n];
        let n: GLsizei = n.try_into().expect("buffer creation count should fit into `GLsizei`");

        unsafe {
            gl::CreateBuffers(n, names.as_mut_ptr());
        }

        names.into_iter().map(|name| Self { name }).collect()
    }

    /// Creates a new buffer object and immediately initializes its data store.
    ///
    /// This is equivalent to calling [`new`][Self::new] and [`set_data`][Self::set_data].
    pub fn new_with_data<'a, T: Into<RawData<'a>>>(data: T, usage: DataUsage) -> Self {
        let mut buffer = Self::new();
        buffer.set_data(data, usage);
        buffer
    }

    /// Binds a buffer object to the specified target.
    ///
    /// This function maps to [`glBindBuffer`].
    ///
    /// [`glBindBuffer`]: https://registry.khronos.org/OpenGL-Refpages/gl4/html/glBindBuffer.xhtml
    pub fn bind(&self, target: BufferTarget) {
        unsafe {
            gl::BindBuffer(target.into(), self.name);
        }
    }

    /// Binds a buffer to an indexed binding target.
    ///
    /// This function maps to [`glBindBufferBase`].
    ///
    /// Calling this function is equivalent to calling [`bind_range`][Self::bind_range] with an `offset` of zero and
    /// `size` equal to the size of the buffer.
    ///
    /// # Panics
    ///
    /// * If `index` does not fit into a [`GLuint`].
    ///
    /// On most systems, this should never happen, since the GL types are almost always equivalent to the given Rust
    /// types.
    ///
    /// [`glBindBufferBase`]: https://registry.khronos.org/OpenGL-Refpages/gl4/html/glBindBufferBase.xhtml
    pub fn bind_base(&self, target: BufferTarget, index: u32) {
        // `GLuint` is almost always guaranteed to be a `u32`, but it can't hurt to check.
        let index: GLuint = index.try_into().expect("buffer index should fit into `GLuint`");
        unsafe {
            gl::BindBufferBase(target.into(), index, self.name);
        }
    }

    /// Binds a range within a buffer o an indexed buffer target.
    ///
    /// This function maps to [`glBindBufferRange`].
    ///
    /// # Panics
    ///
    /// * If `index` does not fit into a [`GLuint`].
    /// * If `offset` does not fit into a [`GLintptr`].
    /// * If `size` does not fit into a [`GLsizeiptr`].
    ///
    /// On most systems, this should never happen, since the GL types are almost always equivalent to the given Rust
    /// types.
    ///
    /// [`glBindBufferRange`]: https://registry.khronos.org/OpenGL-Refpages/gl4/html/glBindBufferRange.xhtml
    pub fn bind_range(&self, target: BufferTarget, index: u32, offset: isize, size: usize) {
        // Again, these should all fit, but we can't always be too sure.
        let index: GLuint = index.try_into().expect("buffer index should fit into `GLuint`");
        let offset: GLintptr = offset.try_into().expect("offset should fit into `GLintptr`");
        let size: GLsizeiptr = size.try_into().expect("size should fit into `GLsizeiptr`");

        unsafe {
            gl::BindBufferRange(target.into(), index, self.name, offset, size);
        }
    }

    /// Creates and initializes a buffer's data store.
    ///
    /// This function maps to [`glNamedBufferData`] (available since OpenGL 4.5), _not_ [`glBufferData`]. This allows it
    /// to be used even when the given buffer is not currently bound.
    ///
    /// [`glBufferData`]: https://registry.khronos.org/OpenGL-Refpages/gl4/html/glBufferData.xhtml
    /// [`glNamedBufferData`]: https://registry.khronos.org/OpenGL-Refpages/gl4/html/glBufferData.xhtml
    ///
    /// # Panics
    ///
    /// * If `data` is too long: its size must fit inside of a [`GLsizeiptr`], which is _usually_ an [`isize`].
    pub fn set_data<'a, T: Into<RawData<'a>>>(&mut self, data: T, usage: DataUsage) {
        let usage = usage.into();

        let data: RawData = data.into();
        let data: &[u8] = data.as_ref();

        let size: GLsizeiptr = data.len().try_into().expect("data size should fit into `GLsizeiptr`");

        unsafe {
            gl::NamedBufferData(self.name, size, data.as_ptr().cast(), usage);
        }
    }

    /// Creates a buffer's data store with uninitialized data.
    ///
    /// This function maps to [`glNamedBufferData`], relying on its special case that states that
    ///
    /// > If _data_ is `NULL`, a data store of the specified is still created, but its contents remain uninitialized and
    /// > thus undefined.
    ///
    /// [`glNamedBufferData`]: https://registry.khronos.org/OpenGL-Refpages/gl4/html/glBufferData.xhtml
    ///
    /// # Panics
    ///
    /// * If `size` is does not fit inside of a [`GLsizeiptr`], which is _usually_ an [`isize`]. A [`usize`] is used in
    ///   the function signature instead so that the caller does not have to convert sizes every time.
    pub fn preallocate(&mut self, size: usize, usage: DataUsage) {
        let usage = usage.into();
        let size: GLsizeiptr = size.try_into().expect("data size should fit into `Glsizeiptr`");
        unsafe {
            gl::NamedBufferData(self.name, size, std::ptr::null(), usage);
        }
    }

    /// Updates a subset of a buffer's data store.
    ///
    /// This function maps to [`glNamedBufferSubData`] (available since OpenGL 4.5), _not_ [`glBufferSubData`]. This
    /// allows it to be used even when the given buffer is not currently bound.
    ///
    /// [`glBufferSubData`]: https://registry.khronos.org/OpenGL-Refpages/gl4/html/glBufferSubData.xhtml
    /// [`glNamedBufferSubData`]: https://registry.khronos.org/OpenGL-Refpages/gl4/html/glBufferSubData.xhtml
    ///
    /// # Panics
    ///
    /// * If `offset` does not fit into a [`GLintptr`].
    /// * If `data`'s size does not fit into a [`GLsizeiptr`].
    pub fn set_sub_data<'a, T: Into<RawData<'a>>>(&mut self, offset: isize, data: T) {
        let data: RawData = data.into();
        let data: &[u8] = data.as_ref();

        let offset: GLintptr = offset.try_into().expect("offset should fit into `GLintptr`");
        let size: GLsizeiptr = data.len().try_into().expect("data size should fit into `GLsizeiptr`");

        unsafe {
            gl::NamedBufferSubData(self.name, offset, size, data.as_ptr().cast());
        }
    }
}

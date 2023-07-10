use gl::types::*;

use crate::{gl_enum, RawData};


gl_enum! {
    /// Acceptable values for [buffer][Buffer] binding targets.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub enum BufferTarget {
        /// Buffer target for vertex attributes.
        ArrayBuffer => ARRAY_BUFFER,
        /// Buffer target for atomic counter storage.
        AtomicCounterBuffer => ATOMIC_COUNTER_BUFFER,
        /// Buffer target for the source of buffer copies.
        CopyReadBuffer => COPY_READ_BUFFER,
        /// Buffer target for the destination of buffer copies.
        CopyWriteBuffer => COPY_WRITE_BUFFER,
        /// Buffer target for indirect compute dispatch commands.
        DispatchIndirectBuffer => DISPATCH_INDIRECT_BUFFER,
        /// Buffer target for indirect command arguments.
        DrawIndirectBuffer => DRAW_INDIRECT_BUFFER,
        /// Buffer target for vertex array indices.
        ElementArrayBuffer => ELEMENT_ARRAY_BUFFER,
        /// Buffer target for the destination of pixel read operations.
        PixelPackBuffer => PIXEL_PACK_BUFFER,
        /// Buffer target for the source of texture data.
        PixelUnpackBuffer => PIXEL_UNPACK_BUFFER,
        /// Buffer target for the query results.
        QueryBuffer => QUERY_BUFFER,
        /// Buffer target for read-write storage for shaders.
        ShaderStorageBuffer => SHADER_STORAGE_BUFFER,
        /// Buffer target for texture data.
        TextureBuffer => TEXTURE_BUFFER,
        /// Buffer target for transform feedback data.
        TransformFeedbackBuffer => TRANSFORM_FEEDBACK_BUFFER,
        /// Buffer target for uniform block storage.
        UniformBuffer => UNIFORM_BUFFER,
    }
}

gl_enum! {
    /// Acceptable usage patterns for [buffer][Buffer] data stores.
    ///
    /// Each value can be broken down into two parts: the frequency of access and the nature of access. The frequency of
    /// access may be one of:
    ///
    /// - **`Stream:`** the data store contents will be modified once and used at most a few times.
    /// - **`Static:`** the data store contents will be modified once and used many times.
    /// - **`Dynamic:`** the data store contents will be modified repeatedly and used many times.
    ///
    /// And the nature of access may be one of:
    ///
    /// - **`Draw:`** the data store contents are modified by the application, and used as the source for GL drawing and
    ///   image specification commands.
    /// - **`Read:`** the data store contents are modified by reading data from the GL, and used to return that data when
    ///   queried by the application.
    /// - **`Copy:`** the data store contents are modified by reading data from the GL, and used as the source for GL
    ///   drawing and image specification commands.
    ///
    /// For example, [`StaticDraw`][Self::StaticDraw], which one might use for vertex data, means that the buffer's data
    /// store will be written into only once, and that it should be used as a source for GL drawing.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub enum BufferUsage {
        StreamDraw => STREAM_DRAW,
        StreamRead => STREAM_READ,
        StreamCopy => STREAM_COPY,
        StaticDraw => STATIC_DRAW,
        StaticRead => STATIC_READ,
        StaticCopy => STATIC_COPY,
        DynamicDraw => DYNAMIC_DRAW,
        DynamicRead => DYNAMIC_READ,
        DynamicCopy => DYNAMIC_COPY,
    }
}


/// An arbitrary buffer in OpenGL.
#[derive(Debug)]
pub struct Buffer {
    pub(crate) name: GLuint,
}

impl Drop for Buffer {
    fn drop(&mut self) {
        unsafe { gl::DeleteBuffers(1, &self.name) }
    }
}

impl Buffer {
    /// Creates a new buffer object.
    ///
    /// This function maps to [`glCreateBuffers`] with a count of one.
    ///
    /// [`glCreateBuffers`]: https://registry.khronos.org/OpenGL-Refpages/gl4/html/glCreateBuffers.xhtml
    pub fn new() -> Self {
        let mut name = 0;

        unsafe {
            gl::CreateBuffers(1, &mut name);
        }

        Self { name }
    }

    /// Creates one or more buffer objects.
    ///
    /// This function panics if `n` is zero.
    ///
    /// This function maps to [`glCreateBuffers`]
    ///
    /// [`glCreateBuffers`]: https://registry.khronos.org/OpenGL-Refpages/gl4/html/glCreateBuffers.xhtml
    pub fn new_multiple(n: usize) -> Vec<Self> {
        assert!(n > 0, "cannot create zero buffer objects");

        let mut names = vec![0; n];
        let n: GLsizei = n.try_into().expect("buffer creation count should fit into `GLsizei`");

        unsafe {
            gl::CreateBuffers(n, names.as_mut_ptr());
        }

        names.into_iter().map(|name| Self { name }).collect()
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
    /// Calling this function is equivalent to calling [`bind_range`][Self::bind_range] with an `offset` of zero and
    /// `size` equal to the size of the buffer.
    ///
    /// This function panics if `index` does not fit into a [`GLuint`]. On most systems, this should never happen, since
    /// the GL types are almost always equivalent to the given Rust types.
    ///
    /// This function maps to [`glBindBufferBase`].
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
    /// This function panics if:
    ///
    /// - `index` does not fit into a [`GLuint`],
    /// - `offset` does not fit into a [`GLintptr`], or
    /// - `size` does not fit into a [`GLsizeiptr`].
    ///
    /// On most systems, this should never happen, since the GL types are almost always equivalent to the given Rust
    /// types.
    ///
    /// This function maps to [`glBindBufferRange`].
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

    /// Creates and initializes the data store for this buffer.
    ///
    /// This function panics if `data` is too long: its size must fit inside of a [`GLsizeiptr`], which is _usually_ an
    /// [`isize`].
    ///
    /// This function maps to [`glNamedBufferData`] (available since OpenGL 4.5), _not_ [`glBufferData`]. This allows it
    /// to be used even when the given buffer is not currently bound.
    ///
    /// [`glBufferData`]: https://registry.khronos.org/OpenGL-Refpages/gl4/html/glBufferData.xhtml
    /// [`glNamedBufferData`]: https://registry.khronos.org/OpenGL-Refpages/gl4/html/glBufferData.xhtml
    pub fn set_data<'a, T: Into<RawData<'a>>>(&mut self, data: T, usage: BufferUsage) {
        let data: RawData = data.into();
        let data: &[u8] = data.as_ref();
        let size: GLsizeiptr = data.len().try_into().expect("data size should fit into `GLsizeiptr`");
        unsafe {
            gl::NamedBufferData(self.name, size, data.as_ptr().cast(), usage.into());
        }
    }

    /// Creates and initializes the data store for the buffer **currently bound** at the given target.
    ///
    /// This function panics if `data` is too long: its size must fit inside of a [`GLsizeiptr`], which is _usually_ an
    /// [`isize`].
    ///
    /// This function maps to [`glBufferData`]. If using OpenGL 4.5 or above, you should prefer using [`Self::set_data`]
    /// instead, which bypasses the binding step by using [`glNamedBufferData`].
    ///
    /// [`glBufferData`]: https://registry.khronos.org/OpenGL-Refpages/gl4/html/glBufferData.xhtml
    /// [`glNamedBufferData`]: https://registry.khronos.org/OpenGL-Refpages/gl4/html/glBufferData.xhtml
    pub fn set_bound_data<'a, T: Into<RawData<'a>>>(target: BufferTarget, data: T, usage: BufferUsage) {
        let data: RawData = data.into();
        let data: &[u8] = data.as_ref();
        let size: GLsizeiptr = data.len().try_into().expect("data size should fit into `GLsizeiptr`");
        unsafe { gl::BufferData(target.into(), size, data.as_ptr().cast(), usage.into()) }
    }

    /// Creates the data store with for this buffer with uninitialized data.
    ///
    /// This function panics if `size` is does not fit inside of a [`GLsizeiptr`], which is _usually_ an [`isize`]. A
    /// [`usize`] is used in the function signature instead so that the caller does not have to convert sizes every
    /// time.
    ///
    /// This function maps to [`glNamedBufferData`], relying on its special case that states that
    ///
    /// > If _data_ is `NULL`, a data store of the specified is still created, but its contents remain uninitialized and
    /// > thus undefined.
    ///
    /// [`glNamedBufferData`]: https://registry.khronos.org/OpenGL-Refpages/gl4/html/glBufferData.xhtml
    pub fn preallocate(&mut self, size: usize, usage: BufferUsage) {
        let size: GLsizeiptr = size.try_into().expect("data size should fit into `GLsizeiptr`");
        unsafe {
            gl::NamedBufferData(self.name, size, std::ptr::null(), usage.into());
        }
    }

    /// Creates the data store for the buffer **currently bound** at the given target with uninitialized data.
    ///
    /// This function panics if `size` is does not fit inside of a [`GLsizeiptr`], which is _usually_ an [`isize`]. A
    /// [`usize`] is used in the function signature instead so that the caller does not have to convert sizes every
    /// time.
    ///
    /// This function maps to [`glBufferData`], relying on its special case that states that
    ///
    /// > If _data_ is `NULL`, a data store of the specified is still created, but its contents remain uninitialized and
    /// > thus undefined.
    ///
    /// If using OpenGL 4.5 or above, you should prefer using [`Self::preallocate`] instead, which bypasses the binding
    /// step by using [`glNamedBufferData`].
    ///
    /// [`glBufferData`]: https://registry.khronos.org/OpenGL-Refpages/gl4/html/glBufferData.xhtml
    pub fn preallocate_bound(target: BufferTarget, size: usize, usage: BufferUsage) {
        let size: GLsizeiptr = size.try_into().expect("data size should fit into `GLsizeiptr`");
        unsafe {
            gl::BufferData(target.into(), size, std::ptr::null(), usage.into());
        }
    }

    /// Updates a subset of this buffer's data store.
    ///
    /// This function panics if:
    ///
    /// - `offset` does not fit into a [`GLintptr`], or
    /// - `data`'s size does not fit into a [`GLsizeiptr`].
    ///
    /// This function maps to [`glNamedBufferSubData`] (available since OpenGL 4.5), _not_ [`glBufferSubData`]. This
    /// allows it to be used even when the given buffer is not currently bound.
    ///
    /// [`glBufferSubData`]: https://registry.khronos.org/OpenGL-Refpages/gl4/html/glBufferSubData.xhtml
    /// [`glNamedBufferSubData`]: https://registry.khronos.org/OpenGL-Refpages/gl4/html/glBufferSubData.xhtml
    pub fn set_sub_data<'a, T: Into<RawData<'a>>>(&mut self, offset: isize, data: T) {
        let data: RawData = data.into();
        let data: &[u8] = data.as_ref();
        let offset: GLintptr = offset.try_into().expect("offset should fit into `GLintptr`");
        let size: GLsizeiptr = data.len().try_into().expect("data size should fit into `GLsizeiptr`");
        unsafe {
            gl::NamedBufferSubData(self.name, offset, size, data.as_ptr().cast());
        }
    }

    /// Updates a subset of the data store for the buffer **currently bound** at the given target.
    ///
    /// This function panics if:
    ///
    /// - `offset` does not fit into a [`GLintptr`], or
    /// - `data`'s size does not fit into a [`GLsizeiptr`].
    ///
    /// This function maps to [`glBufferSubData`]. If using OpenGL 4.5 or above, you should prefer using
    /// [`Self::set_sub_data`] instead, which bypasses the binding step by using [`glNamedBufferSubData`].
    ///
    /// [`glBufferSubData`]: https://registry.khronos.org/OpenGL-Refpages/gl4/html/glBufferSubData.xhtml
    /// [`glNamedBufferSubData`]: https://registry.khronos.org/OpenGL-Refpages/gl4/html/glBufferSubData.xhtml
    pub fn set_bound_sub_data<'a, T: Into<RawData<'a>>>(target: BufferTarget, offset: isize, data: T) {
        let data: RawData = data.into();
        let data: &[u8] = data.as_ref();
        let offset: GLintptr = offset.try_into().expect("offset should fit into `GLintptr`");
        let size: GLsizeiptr = data.len().try_into().expect("data size should fit into `GLsizeiptr`");
        unsafe {
            gl::BufferSubData(target.into(), offset, size, data.as_ptr().cast());
        }
    }
}

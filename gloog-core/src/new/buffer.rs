use std::mem::{self, MaybeUninit};
use std::slice::SliceIndex;

use crate::bindings::types::*;
use crate::{bindings, convert, gl_bitfield, gl_enum, GLContext};

// doc comments link to currently undefined items on purpose; doc warnings will give a checklist for me later :)

gl_enum! {
    /// Usage patterns for buffer data stores.
    ///
    /// Usage differs depending on when buffer data is specified and when it is sourced.
    ///
    /// A buffer's data store is _sourced_ when it is read from as a result of GL commands which specify images, or as a
    /// result of GL commands which invoke shaders accessing buffer data as a result of drawing commands or compute
    /// shader dispatch.
    pub enum BufferUsage {
        /// The data store contents will be specified once by the application, and sourced at most a few times.
        StreamDraw => STREAM_DRAW,
        /// The data store contents will be specified once by reading data from the GL, and queried at most a few times
        /// by the application.
        StreamRead => STREAM_READ,
        /// The data store contents will be specified once by reading data from the GL, and sourced at most a few times.
        StreamCopy => STREAM_COPY,
        /// The data store contents will be specified once by the application, and sourced many times.
        StaticDraw => STATIC_DRAW,
        /// The data store contents will be specified once by reading data from the GL, and queried many times by the
        /// application.
        StaticRead => STATIC_READ,
        ///  The data store contents will be specified once by reading data from the GL, and sourced many times.
        StaticCopy => STATIC_COPY,
        /// The data store contents will be respecified repeatedly by the application, and sourced many times.
        DynamicDraw => DYNAMIC_DRAW,
        /// The data store contents will be respecified repeatedly by reading data from the GL, and queried many times
        /// by the application
        DynamicRead => DYNAMIC_READ,
        /// The data store contents will be respecified repeatedly by reading data from the GL, and sourced many times.
        DynamicCopy => DYNAMIC_COPY,
    }
}

gl_enum! {
    /// Buffer binding targets. See Table 6.1 in the [4.6 core specification][spec].
    ///
    /// [spec]: https://registry.khronos.org/OpenGL/specs/gl/glspec46.core.pdf
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

gl_bitfield! {
    /// Flags describing the intended usage of a [buffer object's data stores][Buffer::buffer_storage].
    pub struct BufferStorageFlags {
        /// The contents of the data store may be updated after creation through calls to [`Buffer::sub_data`]. If this
        /// bit is not set, the buffer content may not be directly updated by the client. The `data` argument may be
        /// used to specify the initial content of the buffer's data store regardless of the presence of the
        /// `DYNAMIC_STORAGE_BIT`. Regardless of the presence of this bit, buffers may always be updated with
        /// server-side calls such as [`Buffer::copy_sub_data`] and [`Buffer::clear_sub_data`].
        pub const DYNAMIC_STORAGE = DYNAMIC_STORAGE_BIT;
        /// The data store may be mapped by the client for read access and a pointer in the client's address space
        /// obtained that may be read from.
        pub const MAP_READ = MAP_READ_BIT;
        /// The data store may be mapped by the client for write access and a pointer in the client's address space
        /// obtained that may be written to
        pub const MAP_WRITE = MAP_WRITE_BIT;
        /// The client may request that the server read from or write to the buffer while it is mapped. The client's
        /// pointer to the data store remains valid so long as the data store is mapped, even during execution of
        /// drawing or dispatch commands.
        pub const MAP_PERSISTENT = MAP_PERSISTENT_BIT;
        /// Shared access to buffers that are simultaneously mapped for client access and are used by the server will be
        /// coherent, so long as that mapping is performed using [`Buffer::map_range`]. That is, data written to the
        /// store by either the client or server will be visible to any subsequently issued GL commands with no further
        /// action taken by the application. In particular,
        ///
        /// * If `MAP_COHERENT_BIT` is not set and the client performs a write followed by a call to one of the
        ///   [`FlushMapped*BufferRange`] commands with a range including the written range, then in subsequent commands
        ///   the server will see the writes.
        ///
        /// * If `MAP_COHERENT_BIT` is set and the client performs a write, then in subsequent commands the server will
        ///   see the writes.
        ///
        /// * If `MAP_COHERENT_BIT` is not set and the server performs a write, the application must call
        ///   `MemoryBarrier` with the `CLIENT_MAPPED_`-`BUFFER_BARRIER_BIT` set and then call FenceSync with
        ///   `SYNC_`-`GPU_COMMANDS_COMPLETE` (or [`finish`][fn@finish]). Then the CPU will see the writes after the
        ///   sync is complete.
        ///
        /// * If `MAP_COHERENT_BIT` is set and the server does a write, the application must call FenceSync with
        ///   SYNC_GPU_COMMANDS_COMPLETE (or [`finish`][fn@finish]). Then the CPU will see the writes after the sync is
        ///   complete.
        pub const MAP_COHERENT = MAP_COHERENT_BIT;
        /// When all other criteria for the buffer storage allocation are met, this bit may be used by an implementation
        /// to determine whether to use storage that is local to the server or to the client to serve as the backing
        /// store for the buffer.
        pub const CLIENT_STORAGE = CLIENT_STORAGE_BIT;
    }
}

gl_bitfield! {
    pub struct BufferAccessFlags {
        /// Indicates that the returned pointer may be used to read buffer object data. No GL error is generated if the
        /// pointer is used to query a mapping which excludes this flag, but the result is undefined and system errors
        /// (possibly including program termination) may occur
        pub const MAP_READ = MAP_READ_BIT;
        /// Indicates that the returned pointer may be used to modify buffer object data. No GL error is generated if
        /// the pointer is used to modify a mapping which excludes this flag, but the result is undefined and system
        /// errors (possibly including program termination) may occur.
        pub const MAP_WRITE = MAP_WRITE_BIT;
        /// Indicates that it is not an error for the GL to read data from or write data to the buffer while it is
        /// mapped (see section 6.3.2). If this bit is set, the value of `BUFFER_STORAGE_FLAGS` for the buffer being
        /// mapped must include `MAP_PERSISTENT_BIT`.
        pub const MAP_PERSISTENT = MAP_PERSISTENT_BIT;
        /// Indicates that the mapping should be performed coherently. That is, such a mapping follows the rules set
        /// forth in section 6.2. If this bit is set, the value of `BUFFER_STORAGE_FLAGS` for the buffer being mapped
        /// must include `MAP_COHERENT_BIT`.
        pub const MAP_COHERENT = MAP_COHERENT_BIT;
        /// Indicates that the previous contents of the specified range may be discarded. Data within this range are
        /// undefined with the exception of subsequently written data. No GL error is generated if subsequent GL
        /// operations access unwritten data, but the result is undefined and system errors (possibly including program
        /// termination) may occur. This flag may not be used in combination with `MAP_READ_BIT`.
        pub const MAP_INVALIDATE_RANGE = MAP_INVALIDATE_RANGE_BIT;
        /// Indicates that the previous contents of the entire buffer may be discarded. Data within the entire buffer
        /// are undefined with the exception of subsequently written data. No GL error is generated if subsequent GL
        /// operations access unwritten data, but the result is undefined and system errors (possibly including program
        /// termination) may occur. This flag may not be used in combination with `MAP_READ_BIT`.
        pub const MAP_INVALIDATE_BUFFER = MAP_INVALIDATE_BUFFER_BIT;
        /// Indicates that one or more discrete subranges of the mapping may be modified. When this flag is set,
        /// modifications to each subrange must be explicitly flushed by calling [`FlushMappedBufferRange`]. No GL error
        /// is set if a subrange of the mapping is modified and not flushed, but data within the corresponding subrange
        /// of the buffer are undefined. This flag may only be used in conjunction with `MAP_WRITE_BIT`. When this
        /// option is selected, flushing is strictly limited to regions that are explicitly indicated with calls to
        /// [`FlushMappedBufferRange`] prior to unmap; if this option is not selected [`UnmapBuffer`] will automatically
        /// flush the entire mapped range when called.
        pub const MAP_FLUSH_EXPLICIT = MAP_FLUSH_EXPLICIT_BIT;
        /// Indicates that the GL should not attempt to synchronize pending operations on the buffer prior to returning
        /// from [`Map*BufferRange`]. No GL error is generated if pending operations which source or modify the buffer
        /// overlap the mapped region, but the result of such previous and any subsequent operations is undefined.
        pub const MAP_UNSYNCHRONIZED = MAP_UNSYNCHRONIZED_BIT;
    }
}


pub struct Buffer<'gl> {
    /// Pointers for OpenGL
    gl: &'gl bindings::GLPointers,
    /// The "name" of this buffer object in OpenGL.
    id: GLuint,
    /// How large this buffer's data store is.
    size: usize,
    /// Information on the buffer's state.
    flags: BufferStorageFlags,
    /// Whether or not this buffer object is mutable.
    is_immutable: bool,
}


impl<'gl> Buffer<'gl> {
    pub const fn size(&self) -> usize {
        self.size
    }

    pub const fn is_immutable(&self) -> bool {
        self.is_immutable
    }

    pub const fn flags(&self) -> BufferStorageFlags {
        self.flags
    }

    /// Create a new empty buffer.
    pub fn empty(ctx: &'gl GLContext) -> Buffer<'gl> {
        let mut name = 0;
        unsafe {
            ctx.gl.create_buffers(1, &mut name);
        }

        Self {
            id: name,
            gl: &ctx.gl,
            size: 0,
            is_immutable: true,
            flags: BufferStorageFlags::none(),
        }
    }


    pub fn storage<T>(&mut self, data: &[T], flags: BufferStorageFlags) {
        let length = convert!(mem::size_of_val(data), GLsizeiptr, "size of buffer data");
        let data = data.as_ptr().cast();

        unsafe {
            self.gl.named_buffer_storage(self.id, length, data, flags.into_raw());
        }

        self.size = length as usize; // fine to cast since we just try_into()'d from a usize to begin with
        self.is_immutable = true; // bufferStorage creates immutable data stores
    }


    pub fn data<T>(&mut self, data: &[T], usage: BufferUsage) {
        let length = convert!(mem::size_of_val(data), GLsizeiptr, "size of buffer data");
        let data = data.as_ptr().cast();

        unsafe {
            self.gl.named_buffer_data(self.id, length, data, usage.into_raw());
        }

        self.size = length as usize;
        self.is_immutable = false; // bufferData creates mutable data stores
    }


    pub fn sub_data<T>(&mut self, offset: usize, data: &[T]) {
        let length = convert!(mem::size_of_val(data), GLsizeiptr, "buffer sub-data size");
        let offset = convert!(offset, GLintptr, "buffer sub-data offset");

        // Do some quick error-checking (4.6 spec, May 5 2022 ver, page 71)
        if offset + length > self.size as isize {
            panic!("buffer range out of bounds");
        } else if self.is_immutable && !self.flags.contains(BufferStorageFlags::DYNAMIC_STORAGE) {
            panic!("attempt to modify immutable buffer storage");
        }

        // ----

        let data = data.as_ptr().cast();
        unsafe {
            self.gl.named_buffer_sub_data(self.id, offset, length, data);
        }
    }


    // TODO: `clearBufferSubData`. Defining enums and stuff looks like it'll be a challenge. Section 8.4.4 is a bit hard
    // to follow. Considering making a trait and implementing it for things like `[u8; 4]` etc. to be able to statically
    // determine which `type` and `format` parameters to use.

    // TODO: maybe a pair of `map_range_mut` and `map_range` functions? would fit well with Rust conventions.

    /// # Safety
    ///
    /// The caller must ensure that the mapped range is aligned for `T`. That is, that whatever data is already in the
    /// buffer starting at `offset` is properly aligned with
    pub unsafe fn map_range<'a, T>(
        &'a mut self,
        offset: usize,
        length: usize,
        access: BufferAccessFlags,
    ) -> MappedBuffer<'gl, 'a, T> {
        let offset = convert!(offset, GLintptr, "mapped buffer offset");
        let length = convert!(length, GLsizeiptr, "mapped buffer length");

        // TODO -- write safety comment lol
        let map_slice: &mut [MaybeUninit<T>] = unsafe {
            let ptr = self.gl.map_named_buffer_range(self.id, offset, length, access.into_raw());
            std::slice::from_raw_parts_mut(ptr.cast(), length as usize)
        };

        MappedBuffer {
            buf: self,
            mapped_slice: map_slice,
            map_offset: offset as usize,
            map_length: length as usize,
        }
    }
}


impl<'gl> Drop for Buffer<'gl> {
    fn drop(&mut self) {
        unsafe {
            self.gl.delete_buffers(1, &self.id);
        }
    }
}


// TODO: would it be better to just map it as a raw `[u8]`? Then there are no invariants for us to worry about; caller
// would use bytemuck or Read/Write traits to get data in and out.

// ^ yeah probably TODO TODO

pub struct MappedBuffer<'gl, 'buf, T> {
    /// The original buffer object this mapping came from.
    buf: &'buf mut Buffer<'gl>,
    /// The offset, in bytes, into the original buffer at which this map lies.
    map_offset: usize,
    /// How much of the buffer, in bytes, this mapping takes up.
    map_length: usize,
    /// The mapped memory.
    mapped_slice: &'buf mut [MaybeUninit<T>],
}


impl<'gl, 'buf, T> MappedBuffer<'gl, 'buf, T> {
    pub fn unmap(self) -> bool {
        unsafe { self.buf.gl.unmap_named_buffer(self.buf.id) == bindings::TRUE }
    }

    pub fn len(&self) -> usize {
        self.mapped_slice.len()
    }

    pub fn offset(&self) -> usize {
        // urrgh... do I wanna store both sizes? do I wanna store it as multiples of T, and multiply for bytes? or do I
        // wanna do this division? division is stinky but I'm not sure which `len` or `offset` I'll need more often.
        // I'll leave this for now.
        self.map_offset / mem::size_of::<T>()
    }

    pub fn len_bytes(&self) -> usize {
        self.map_length
    }

    pub fn offset_bytes(&self) -> usize {
        self.map_offset
    }
}


impl<'gl, 'buf, T> Drop for MappedBuffer<'gl, 'buf, T> {
    fn drop(&mut self) {
        unsafe {
            self.buf.gl.unmap_named_buffer(self.buf.id);
        }
    }
}


// Implement indexing on MappedBuffer for any type that can already index into a slice:

impl<'gl, 'buf, T, I> std::ops::Index<I> for MappedBuffer<'gl, 'buf, T>
where
    I: SliceIndex<[MaybeUninit<T>]>,
{
    type Output = I::Output;

    fn index(&self, index: I) -> &Self::Output {
        &self.mapped_slice[index]
    }
}

impl<'gl, 'buf, T, I> std::ops::IndexMut<I> for MappedBuffer<'gl, 'buf, T>
where
    I: SliceIndex<[MaybeUninit<T>]>,
{
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        &mut self.mapped_slice[index]
    }
}

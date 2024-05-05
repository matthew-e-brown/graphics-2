use std::ffi::c_void;
use std::mem::ManuallyDrop;
use std::rc::Rc;
use std::{mem, ptr};

use crate::raw::types::*;
use crate::raw::GLPointers;
use crate::{convert, gl_bitfield, gl_enum, GLContext};


// [TODO] Decide how we're going to handle GL errors. Do we anticipate all the error conditions and panic? Do we make
// functions return `Result<()>`? Right now we just let them error. Will implement a function to get errors sooner or
// later.

// [TODO] Make better use of type-level abstractions for different stages of buffer allocation. As in,


gl_enum! {
    /// Usage patterns for buffer object data stores.
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
    /// Access modes for mapped buffers.
    pub enum BufferAccess {
        ReadOnly => READ_ONLY,
        WriteOnly => WRITE_ONLY,
        ReadWrite => READ_WRITE,
    }
}

gl_bitfield! {
    /// Flags describing the intended usage of a [buffer object's data store][Buffer::buffer_storage].
    pub struct BufferStorageFlags {
        /// The contents of the data store may be updated after creation through calls to [`Buffer::sub_data`]. If this
        /// bit is not set, the buffer content may not be directly updated by the client. The `data` argument may be
        /// used to specify the initial content of the buffer's data store regardless of the presence of the
        /// `DYNAMIC_STORAGE` bit. Regardless of the presence of this bit, buffers may always be updated with
        /// server-side calls such as [`Buffer::copy_sub_data`] and [`Buffer::clear_sub_data`].
        pub const DYNAMIC_STORAGE = DYNAMIC_STORAGE_BIT;

        /// The data store may be mapped by the client for read access and a pointer in the client's address space
        /// obtained that may be read from.
        pub const MAP_READ = MAP_READ_BIT;

        /// The data store may be mapped by the client for write access and a pointer in the client's address space
        /// obtained that may be written to.
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
        /// * If the `MAP_COHERENT` bit **is not** set and the client performs a write followed by a call to one of the
        ///   [`FlushMapped*BufferRange`] commands with a range including the written range, then in subsequent commands
        ///   the server will see the writes.
        ///
        /// * If the `MAP_COHERENT` bit **is** set and the client performs a write, then in subsequent commands the
        ///   server will see the writes.
        ///
        /// * If the `MAP_COHERENT` bit **is not** set and the server performs a write, the application must call
        ///   `glMemoryBarrier` with the [`CLIENT_MAPPED_BUFFER_BARRIER`] bit set and then call [`glFenceSync`] with
        ///   `SYNC_GPU_COMMANDS_COMPLETE` (or [`glFinish`]). Then the CPU will see the writes after the sync is
        ///   complete.
        ///
        /// * If the `MAP_COHERENT` bit **is** set and the server does a write, the application must call FenceSync with
        ///   SYNC_GPU_COMMANDS_COMPLETE (or [`glFinish`]). Then the CPU will see the writes after the sync is complete.
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


impl GLContext {
    /// Creates a new buffer object.
    pub fn create_buffer(&self) -> Buffer {
        let mut name: GLuint = 0;
        unsafe {
            self.gl.create_buffers(1, &mut name);
        }

        Buffer {
            gl: Rc::clone(&self.gl),
            id: name,
            state: None,
        }
    }

    /// Creates `n` new buffer objects.
    pub fn create_buffers(&self, n: usize) -> Vec<Buffer> {
        let mut names: Vec<GLuint> = vec![0; n];
        let len = convert!(names.len(), GLsizei, "number of buffers");
        unsafe {
            self.gl.create_buffers(len, names.as_mut_ptr());
        }

        names
            .into_iter()
            .map(|name| Buffer {
                id: name,
                gl: Rc::clone(&self.gl),
                state: None,
            })
            .collect()
    }
}


/// A handle for an OpenGL buffer object.
pub struct Buffer {
    pub(crate) gl: Rc<GLPointers>,
    pub(crate) id: GLuint,
    /// The state of this buffer's data store, if it has been allocated.
    state: Option<BufferState>,
}

impl Drop for Buffer {
    fn drop(&mut self) {
        unsafe { self.gl.delete_buffers(1, &self.id) }
    }
}

/// Server-side state for an OpenGL buffer object.
#[derive(Debug)]
struct BufferState {
    /// The allocated size of the buffer store.
    size: usize,
    /// The usage hint for this buffer store.
    usage: BufferUsage,
    /// Whether or not this buffer object's data store is immutable.
    immutable: bool,
    /// Flags for this buffer object.
    storage_flags: BufferStorageFlags,
    // // Additional state that only applies to mapped buffers:
    // access: BufferAccess,
    // map_pointer: Option<NonNull<c_void>>,
    // map_offset: GLint64,
    // map_length: GLint64,
    // access_flags: BufferAccessFlags,
}


impl Buffer {
    /// Returns a reference to this ID.
    pub const fn id(&self) -> GLuint {
        self.id
    }

    /// Returns `true` if this buffer's data store has been allocated with a call to [`Buffer::storage`] or
    /// [`Buffer::data`].
    pub const fn is_allocated(&self) -> bool {
        self.state.is_some()
    }

    /// Returns `Some(true)` if this buffer is allocated and mutable; `None` if it has not been allocated yet.
    ///
    /// [`Buffer::storage`] creates immutable stores, and [`Buffer::data`] creates mutable ones. A buffer whose store is
    /// allocated with `Buffer::storage` may still have its _contents_ modified if the [`DYNAMIC_STORAGE`] bit is set in
    /// its [storage flags][Buffer::storage_flags]; a buffer is _mutable_ in OpenGL if it may have its data store
    /// re-allocated with a new size throughout its lifetime.
    pub const fn is_mutable(&self) -> Option<bool> {
        match &self.state {
            Some(state) => Some(!state.immutable),
            None => None,
        }
    }

    /// Returns the size of this buffer's data store, if it has been [allocated][Self::is_allocated].
    pub const fn size(&self) -> Option<usize> {
        match &self.state {
            Some(state) => Some(state.size),
            None => None,
        }
    }

    /// Gets this buffer's current usage, if its data store has been [allocated][Self::is_allocated].
    pub const fn usage(&self) -> Option<BufferUsage> {
        match &self.state {
            Some(state) => Some(state.usage),
            None => None,
        }
    }

    /// Gets this buffer's current storage flags, if its data store has been [allocated][Self::is_allocated].
    ///
    /// If the data store has the [`DYNAMIC_STORAGE`] bit set, then it may have its contents updated with calls to
    /// [`Buffer::sub_data`].
    pub const fn storage_flags(&self) -> Option<BufferStorageFlags> {
        match &self.state {
            Some(state) => Some(state.storage_flags),
            None => None,
        }
    }

    /// Deletes one or more buffer objects.
    ///
    /// Normally, a buffer may be deleted by dropping it. This method will drop them all in a single OpenGL call. Note
    /// that this will allocated a vec of [`GLuint`]s to pass to OpenGL, which may be less desirable to some than making
    /// multiple OpenGL calls.
    ///
    /// # Safety
    ///
    /// This function assumes that all the provided buffers belong to the same OpenGL context, or at least that they may
    /// be deleted using the same call to `glDeleteBuffers`.
    pub fn delete<I: IntoIterator<Item = Buffer>>(buffers: I) {
        let mut local_gl = None; // Need a `GLPointers` to actually do the deletion, grab it as we extract IDs
        let names: Vec<GLuint> = buffers
            .into_iter()
            .map(|buffer| {
                // Inhibit the buffer's `Drop` impl
                let buffer = ManuallyDrop::new(buffer);
                // Copy the RC out of each buffer object; this copy will get dropped and properly decrement the refcount
                local_gl = Some(unsafe { ptr::read(&buffer.gl) });
                buffer.id
            })
            .collect();

        if let Some(gl) = local_gl {
            let len = convert!(names.len(), GLsizei, "number of buffers");
            unsafe {
                gl.delete_buffers(len, names.as_ptr());
            }
        }
    }

    /// Allocates an immutable data store for a buffer object.
    ///
    /// This deletes any existing data store for this buffer object and resets its state.
    ///
    /// # Errors
    ///
    /// * If `flags` contains [`MAP_PERSISTENT`], it must also contain at least one of [`MAP_READ`] or [`MAP_WRITE`].
    /// * If `flags` contains [`MAP_COHERENT`], it must also contain [`MAP_PERSISTENT`].
    pub fn storage<T, D: AsRef<[T]>>(&mut self, data: D, flags: BufferStorageFlags) {
        let data = data.as_ref();
        let size = mem::size_of::<T>() * data.len();
        // (NB: mult will never overflow since `data` could never be more than `isize::MAX` in Rust)

        let data_ptr = data.as_ptr().cast::<c_void>();
        let size_sgn = convert!(size, GLsizeiptr, "buffer data size");
        unsafe {
            self.gl.named_buffer_storage(self.id, size_sgn, data_ptr, flags.into_raw());
        }

        // Set this buffer's state according to table 6.3 in the spec
        self.state = Some(BufferState {
            size,
            usage: BufferUsage::DynamicDraw,
            immutable: true,
            storage_flags: flags,
        });
    }

    /// Allocates a mutable data store for a buffer object.
    ///
    /// This deletes any existing data store for this buffer object and resets its state.
    ///
    /// Allocating a buffer with this function sets its [storage flags][Buffer::storage_flags] to the OR of
    /// <code>[MAP_READ] | [MAP_WRITE] | [DYNAMIC_STORAGE]</code>.
    ///
    /// [MAP_READ]: BufferStorageFlags::MAP_READ
    /// [MAP_WRITE]: BufferStorageFlags::MAP_WRITE
    /// [DYNAMIC_STORAGE]: BufferStorageFlags::DYNAMIC_STORAGE
    pub fn data<T, D: AsRef<[T]>>(&mut self, data: D, usage: BufferUsage) {
        let data = data.as_ref();
        let size = mem::size_of::<T>() * data.len();

        let data_ptr = data.as_ptr().cast::<c_void>();
        let size_sgn = convert!(size, GLsizeiptr, "buffer data size");
        unsafe {
            self.gl.named_buffer_data(self.id, size_sgn, data_ptr, usage.into_raw());
        }

        // See table 6.3
        self.state = Some(BufferState {
            size,
            usage,
            immutable: false,
            storage_flags: BufferStorageFlags::MAP_READ
                | BufferStorageFlags::MAP_WRITE
                | BufferStorageFlags::DYNAMIC_STORAGE,
        });
    }

    /// Modifies some or all of the data contained in this buffer object's data store.
    pub fn sub_data<T, D: AsRef<[T]>>(&mut self, offset: usize, data: D) {
        let data = data.as_ref();
        let size = mem::size_of::<T>() * data.len();

        let offset = convert!(offset, GLintptr, "buffer sub data offset");
        let data_ptr = data.as_ptr().cast::<c_void>();
        let size_sgn = convert!(size, GLsizeiptr, "buffer data size");
        unsafe {
            self.gl.named_buffer_sub_data(self.id, offset, size_sgn, data_ptr);
        }
    }

    // [TODO] clear_sub-data

    // [TODO] map (will probably require another type)
}

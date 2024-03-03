use std::ffi::CString;
use std::ptr::from_ref;

use gloog_math::{Mat2, Mat3, Mat4, Vec2, Vec3, Vec4};

use crate::raw::types::*;
use crate::raw::GLPointers;
use crate::types::*;
use crate::{convert, GLContext};


impl GLContext {
    /// A direct wrapper for [`glGetUniformLocation`]. If OpenGL returns `-1` (the uniform was not found in the given
    /// program), this function returns `None`.
    ///
    /// Note that [`UniformLocation`]'s implementation of [`Default`] means that you can use [`unwrap_or_default`] on
    /// the return value of this function to allow the `-1` variant through. Alternatively, you can call
    /// [`get_uniform_location_unchecked`] to avoid the check in the first place.
    ///
    /// Also note that, since Rust strings do not have NUL-terminators, `name` must be copied into a [`CString`] in
    /// order to pass it to OpenGL.
    ///
    /// [`glGetUniformLocation`]: https://registry.khronos.org/OpenGL-Refpages/gl4/html/glGetUniformLocation.xhtml
    /// [`unwrap_or_default`]: Option::unwrap_or_default
    /// [`get_uniform_location_unchecked`]: Self::get_uniform_location_unchecked
    pub fn get_uniform_location(&self, program: ProgramID, name: &str) -> Option<UniformLocation> {
        match self.get_uniform_location_unchecked(program, name) {
            UniformLocation(-1) => None,
            loc => Some(loc),
        }
    }

    /// Just like [`get_uniform_location`][Self::get_uniform_location] except that it does not check for `-1` return
    /// values. This is completely safe, since OpenGL uniform functions are perfectly happy receiving a `-1` location
    /// for uniforms.
    pub fn get_uniform_location_unchecked(&self, program: ProgramID, name: &str) -> UniformLocation {
        let name = CString::new(name).expect("uniform name should not contain NUL-bytes");
        let loc = unsafe { self.gl.get_uniform_location(program.into_raw(), name.as_ptr()) };
        UniformLocation::new(loc)
    }

    /// Calls the appropriate [`glUniform*`] function for a value depending on how that value has implemented the
    /// [`Uniform`] trait.
    ///
    /// # Example
    ///
    /// ```
    /// # const program: ProgramID = ProgramID::new(0);
    /// # fn perspective(_fov_deg: f32, _aspect: f32, _near_clip: f32, _far_clip: f32) -> Mat4 { Mat4::IDENTITY }
    ///
    /// fn draw_loop(gl: &GLContext) {
    ///     // --- snip ---
    ///
    ///     let u_lp_loc = gl.get_uniform_location(program, "uLightPos");
    ///     let light_pos = Vec3 { x: 1.0, y: 2.0, z: 3.0 }; // Implements `Uniform`
    ///
    ///     let u_pm_loc = gl.get_uniform_location(program, "uProjectionMatrix");
    ///     let proj_mat: Mat4 = perspective(45.0, 1.00, 0.25, 50.0); // Also implements `Uniform`
    ///
    ///     gl.uniform(u_lp_loc, &light_pos); // calls `glUniform3fv` under the hood
    ///     gl.uniform(u_pm_loc, &proj_mat); // calls `glUniformMatrix4fv` under the hood
    ///
    ///     // --- snip ---
    /// }
    /// ```
    ///
    /// [`glUniform*`]: https://registry.khronos.org/OpenGL-Refpages/gl4/html/glUniform.xhtml
    pub fn uniform<T: Uniform>(&self, location: UniformLocation, value: &T) {
        // SAFETY: the implementors of `Uniform` are expected to uphold the safety contract.
        unsafe { T::set_uniform(&self.gl, location.into_raw(), value.count(), value.get_ptr()) }
    }
}


/// Represents a value that can be sent to OpenGL as a uniform.
///
///
/// This trait is what powers the [`GLContext::uniform`] method; for the most part, you should neither need to implement
/// it nor call any of its methods. It has already been implemented for all Rust primitive types that map to an OpenGL
/// uniform type ([`f32`], [`i32`], [`u32`], and [`bool`]), as well as all of the basic [`gloog_math`] types.
///
/// # Implementing this trait
///
/// Values that implement this trait need to ensure that the pointer returned by [`get_ptr`] is safe for OpenGL to read
/// from. The _number_ of bytes that it should be safe to read from said pointer is determined by which
/// [`GLPointer`][GLPointers] function is used within [`set_uniform`].
///
/// Generally, implementors of the `set_uniform` method should guarantee that they don't call any function other than
/// one of the `glUniform` functions from [`GLPointers`]. The underlying function used should be able to support more
/// than one value; that is, **it should be one of the `glUniform*v` variants.** This is because there exists a blanket
/// implementation of this trait that covers any slices of uniforms, which rely on the assumption that `T::Uniform` is
/// safe to sound for a `&[T]` (just with a different count).
///
/// The reason that [`get_ptr`] and [`count`] are exposed as separate methods on this trait is so that
/// [`GLContext::uniform`] can use them to determine which values to send to `set_uniform`.
///
/// The implementation of this trait is perhaps best given by an example.
///
/// ```
/// // SAFETY: `gloog_math::Vec3` is `#[repr(C)]`, and so it is guaranteed to be equivalent to an array of three floats.
/// unsafe impl Uniform for Vec3 {
///     type PtrType = gloog_core::raw::types::GLfloat; // equivalent to f32
///
///     fn ptr(&self) -> *const Self::PtrType {
///         // Vec3 has an `as_ptr` method, so we'll use that directly:
///         self.as_ptr()
///     }
///
///     fn count(&self) -> GLsizei {
///         // There are three floats, but that's only 1 of the things that `glUniform3fv` is expecting:
///         1
///     }
///
///     unsafe fn set_uniform(gl: &GLPointers, location: GLint, count: GLsizei, value: *const Self::PtrType) {
///         // Vec3 doesn't need to do anything fancy here, so we just pass through the
///         gl.uniform_3fv(location, count, value)
///     }
/// }
/// ```
///
/// The reason that `set_uniform` is a trait method is so that it can wrap a function that provides a bit more context
/// to the underlying `GLPointers` call. For example, matrices need more than a `count` and a `value` pointer:
///
/// ```
/// unsafe impl Uniform for Mat4 {
///     type PtrType = f32;
///
///     // --- snip ---
///
///     unsafe fn set_uniform(
///         gl: &GLPointers,
///         location: GLint,
///         count: GLsizei,
///         value: *const Self::PtrType,
///     ) {
///         gl.uniform_matrix_4fv(location, count, /* transpose: */ false as GLboolean, value)
///     }
/// }
/// ```
///
/// Now, [`GLContext::uniform`] can call `Mat4::set_uniform` with just a location, count, and value without needing to
/// supply an extra parameter. Otherwise, we would need a way to encapsulate the extra parameter within the trait
/// signature, something that isn't really possible (at least in any nice way).
///
/// [`get_ptr`]: Uniform::get_ptr
/// [`count`]: Uniform::count
/// [`set_uniform`]: Uniform::set_uniform
pub unsafe trait Uniform {
    /// What `GLtype` this value is; used to for pointer casting. For example, [`Vec3`] uses [`GLfloat`].
    type PtrType;

    /// Converts a reference of this type into a pointer. The return value of this function is fed into
    /// [`set_uniform`][`Uniform::set_uniform`] by [`GLContext::uniform`].
    fn get_ptr(&self) -> *const Self::PtrType {
        from_ref(self).cast()
    }

    /// Determines the number of individual values this item is made up of. The return value of this function is fed
    /// into [`set_uniform`][`Uniform::set_uniform`] by [`GLContext::uniform`].
    ///
    /// For most implementations, this function simply returns `1`. It is only greater than 1 when an _array_ of this
    /// type value is being sent; however, there is already a blanket implementation that handles sending a slice of any
    /// `Uniform`-implemented value, so most uses need not worry about this detail.
    fn count(&self) -> GLsizei {
        1
    }

    /// Wrapper for the raw [`GLPointers`] function that should be used to sends one or more uniforms of this type to
    /// OpenGL.
    ///
    /// You should generally not call this method; use [`GLContext::uniform`] instead. That function guarantees that the
    /// value of `count` and `value` will always be the result of [`get_ptr()`] and [`count()`].
    ///
    /// [`get_ptr()`]: Uniform::get_ptr
    /// [`count()`]: Uniform::count
    unsafe fn set_uniform(gl: &GLPointers, location: GLint, count: GLsizei, value: *const Self::PtrType);
}

// ---------------------------------------------------------------------------------------------------------------------
// Implementations
// ---------------------------------------------------------------------------------------------------------------------

#[rustfmt::skip]
macro_rules! uniform_type {
    (f32) => (GLfloat);
    (i32) => (GLint);
    (u32) => (GLuint);
    (bool) => (GLuint);

    ([f32; $n:literal]) => (GLfloat);
    ([i32; $n:literal]) => (GLint);
    ([u32; $n:literal]) => (GLuint);
    ([bool; $n:literal]) => (GLuint);

    (Vec2) => (GLfloat);
    (Vec3) => (GLfloat);
    (Vec4) => (GLfloat);
}

macro_rules! impl_uniform {
    (matrix, $rs_type:ty, $func:ident) => {
        unsafe impl Uniform for $rs_type {
            type PtrType = GLfloat; // Only `fv` matrices

            fn get_ptr(&self) -> *const Self::PtrType {
                self.as_ptr().cast()
            }

            fn count(&self) -> GLsizei {
                1
            }

            unsafe fn set_uniform(gl: &GLPointers, location: GLint, count: GLsizei, value: *const Self::PtrType) {
                unsafe { gl.$func(location, count, false as GLboolean, value) } // false for transpose
            }
        }
    };

    ($ptr_style:tt, $rs_type:tt, $func:ident) => {
        unsafe impl Uniform for $rs_type {
            type PtrType = uniform_type!($rs_type);

            fn get_ptr(&self) -> *const Self::PtrType {
                impl_uniform!(@ $ptr_style, self)
            }

            fn count(&self) -> GLsizei {
                1 // even for arrays; simply sending 1x of [f32; 4], for example
            }

            unsafe fn set_uniform(gl: &GLPointers, location: GLint, count: GLsizei, value: *const Self::PtrType) {
                unsafe { gl.$func(location, count, value) }
            }
        }
    };

    (@ from_ref, $this:ident) => {
        from_ref($this).cast()
    };

    (@ as_ptr, $this:ident) => {
        $this.as_ptr().cast()
    };
}


impl_uniform!(from_ref, f32, uniform_1fv);
impl_uniform!(as_ptr, [f32; 1], uniform_1fv);
impl_uniform!(as_ptr, [f32; 2], uniform_2fv);
impl_uniform!(as_ptr, [f32; 3], uniform_3fv);
impl_uniform!(as_ptr, [f32; 4], uniform_4fv);

impl_uniform!(from_ref, i32, uniform_1iv);
impl_uniform!(as_ptr, [i32; 1], uniform_1iv);
impl_uniform!(as_ptr, [i32; 2], uniform_2iv);
impl_uniform!(as_ptr, [i32; 3], uniform_3iv);
impl_uniform!(as_ptr, [i32; 4], uniform_4iv);

impl_uniform!(from_ref, u32, uniform_1uiv);
impl_uniform!(as_ptr, [u32; 1], uniform_1uiv);
impl_uniform!(as_ptr, [u32; 2], uniform_2uiv);
impl_uniform!(as_ptr, [u32; 3], uniform_3uiv);
impl_uniform!(as_ptr, [u32; 4], uniform_4uiv);

impl_uniform!(from_ref, bool, uniform_1uiv);
impl_uniform!(as_ptr, [bool; 1], uniform_1uiv);
impl_uniform!(as_ptr, [bool; 2], uniform_2uiv);
impl_uniform!(as_ptr, [bool; 3], uniform_3uiv);
impl_uniform!(as_ptr, [bool; 4], uniform_4uiv);

impl_uniform!(as_ptr, Vec2, uniform_2fv);
impl_uniform!(as_ptr, Vec3, uniform_3fv);
impl_uniform!(as_ptr, Vec4, uniform_4fv);

impl_uniform!(matrix, Mat2, uniform_matrix_2fv);
impl_uniform!(matrix, Mat3, uniform_matrix_3fv);
impl_uniform!(matrix, Mat4, uniform_matrix_4fv);

impl_uniform!(matrix, [[f32; 2]; 2], uniform_matrix_2fv);
impl_uniform!(matrix, [[f32; 3]; 3], uniform_matrix_3fv);
impl_uniform!(matrix, [[f32; 4]; 4], uniform_matrix_4fv);


/// Because all uniforms make use of the `Uniform*v` functions, it can be safely implemented it for all slices by simply
/// casting the pointer to the slice.
unsafe impl<T> Uniform for &[T]
where
    T: Uniform,
{
    type PtrType = T::PtrType;

    fn get_ptr(&self) -> *const Self::PtrType {
        <[T]>::as_ptr(self).cast() // cast slice ptr
    }

    fn count(&self) -> GLsizei {
        convert!(self.len(), GLsizei, "number of uniform values")
    }

    unsafe fn set_uniform(gl: &GLPointers, location: GLint, count: GLsizei, ptr: *const Self::PtrType) {
        // To set an array's uniform values, simply use the same wrapper function that the base type defined. The
        // overrides of `count()` and `as_ptr()` will be used for the `count` and `ptr` parameters.
        T::set_uniform(gl, location, count, ptr)
    }
}

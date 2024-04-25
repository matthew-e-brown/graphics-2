use std::ffi::CString;
use std::ops::Deref;
use std::ptr::from_ref;

use gloog_math::{Mat2, Mat3, Mat4, Vec2, Vec3, Vec4};

use crate::raw::types::*;
use crate::raw::GLPointers;
use crate::shader::Program;
use crate::{convert, gl_newtype};


gl_newtype! {
    /// The location of a uniform value in a shader program.
    ///
    /// This type behaves similar to an [`Option`], with `-1` serving as its `None` variant.
    pub struct UniformLocation(GLint);
}

impl UniformLocation {
    /// Check if this uniform was found in the shader program.
    pub const fn is_some(&self) -> bool {
        self.0 != -1
    }

    /// Check if this uniform was not found in the shader program.
    pub const fn is_none(&self) -> bool {
        self.0 == -1
    }
}

impl Default for UniformLocation {
    fn default() -> Self {
        UniformLocation(-1)
    }
}


impl Program {
    /// A direct wrapper for [`glGetUniformLocation`]. If OpenGL returns `-1` (the uniform was not found in the given
    /// program), this function returns `None`.
    ///
    /// Note that [`UniformLocation`]'s implementation of [`Default`] means that you can use [`unwrap_or_default`] on
    /// the return value of this function to allow the `-1` variant through. Alternatively, you can call
    /// [`get_uniform_location_unchecked`] to avoid the check in the first place.
    ///
    /// Also note that, since Rust strings do not have NUL-terminators, `name` is copied into a [`CString`] before
    /// passing it to OpenGL.
    ///
    /// # Panics
    ///
    /// This function panics if `self` has not been linked yet.
    ///
    /// [`glGetUniformLocation`]: https://registry.khronos.org/OpenGL-Refpages/gl4/html/glGetUniformLocation.xhtml
    /// [`unwrap_or_default`]: Option::unwrap_or_default
    /// [`get_uniform_location_unchecked`]: Self::get_uniform_location_unchecked
    pub fn get_uniform_location(&self, name: &str) -> Option<UniformLocation> {
        match self.get_uniform_location_unchecked(name) {
            UniformLocation(-1) => None,
            loc => Some(loc),
        }
    }

    /// Just like [`get_uniform_location`][Self::get_uniform_location] except that it does not check for `-1` return
    /// values. This is completely safe, since OpenGL uniform functions are perfectly happy receiving a `-1` location
    /// for uniforms.
    ///
    /// # Panics
    ///
    /// This function panics if `self` has not been linked yet.
    pub fn get_uniform_location_unchecked(&self, name: &str) -> UniformLocation {
        if !self.is_linked() {
            panic!("attempted to query uniform location in a non-linked shader program object");
        }

        let name = CString::new(name).expect("uniform name should not contain NUL-bytes");
        let loc = unsafe { self.gl.get_uniform_location(self.id, name.as_ptr()) };
        unsafe { UniformLocation::from_raw_unchecked(loc) }
    }

    /// Sets a uniform value in this shader program.
    ///
    /// This method calls a different variant of [`glProgramUniform*`] depending on how the provided value has
    /// implemented the [`Uniform`] trait.
    ///
    /// # Example
    ///
    /// ```
    /// # fn perspective(_fov_deg: f32, _aspect: f32, _near_clip: f32, _far_clip: f32) -> Mat4 { Mat4::IDENTITY }
    ///
    /// fn draw_loop(gl: &GLContext, program: &mut Program) {
    ///     // --- snip ---
    ///
    ///     let u_lp_loc = program.get_uniform_location("uLightPos");
    ///     let light_pos = Vec3 { x: 1.0, y: 2.0, z: 3.0 }; // Implements `Uniform`
    ///
    ///     let u_pm_loc = program.get_uniform_location("uProjectionMatrix");
    ///     let proj_mat: Mat4 = perspective(45.0, 1.00, 0.25, 50.0); // Also implements `Uniform`
    ///
    ///     program.uniform(u_lp_loc, &light_pos); // calls `glProgramUniform3fv` under the hood
    ///     program.uniform(u_pm_loc, &proj_mat); // calls `glProgramUniformMatrix4fv` under the hood
    ///
    ///     // --- snip ---
    /// }
    /// ```
    ///
    /// [`glProgramUniform*`]: https://registry.khronos.org/OpenGL-Refpages/gl4/html/glProgramUniform.xhtml
    ///
    /// NB: this function takes `&self` instead of `&mut self` since it doesn't really modify the _context,_ it modifies
    /// a program.
    pub fn uniform<T: Uniform>(&mut self, location: UniformLocation, value: &T) {
        // SAFETY: the implementors of `Uniform` are expected to uphold the safety contract.
        unsafe { T::set_uniform(self.gl.deref(), self.id, location.into_raw(), value.count(), value.get_ptr()) }
    }
}


/// Represents a value that can be sent to OpenGL as a uniform.
///
/// This trait is what powers the [`Program::uniform`] method. For the most part, you should neither need to implement
/// this trait nor call any of its methods directly. This trait has already been implemented for all Rust primitive
/// types that map to an OpenGL uniform type ([`f32`], [`i32`], [`u32`], and [`bool`]), as well as all of the basic
/// [`gloog_math`] types.
///
///
/// # Arrays and slices
///
/// This trait is automatically implemented for any slice of values that implements `Uniform`. It may be worth noting
/// how this implementation will map down to OpenGL function calls. We'll use floats as an example.
///
/// | Rust Type                                | OpenGL function call        | `count` argument |
/// | :--------------------------------------- | --------------------------: | ---------------: |
/// | A single `f32`                           |       `glProgramUniform1fv` |                1 |
/// | A slice `&[f32]` of length `n`           |       `glProgramUniform1fv` |              `n` |
/// | An array `[f32; 4]`                      |       `glProgramUniform4fv` |                1 |
/// | A 2D array `[[f32; 4]; 4]`               | `glProgramUniformMatrix4fv` |                1 |
/// | A slice `&[[f32; 4]]` of length `n`      |       `glProgramUniform4fv` |              `n` |
/// | A slice `&[[[f32; 4]; 4]]` of length `n` | `glProgramUniformMatrix4fv` |              `n` |
///
/// The OpenGL spec states that the `fv` versions of the `glUniform*` command will set "a float, a floating-point
/// vector, or an array of either of these types," so this implementation should work without issue; but it can't hurt
/// to be aware of the inner workings.
///
///
/// # Implementing this trait
///
/// Values that implement this trait need to ensure that the pointer returned by [`get_ptr`] is safe for OpenGL to read
/// from. The _number_ of bytes that should be safe to read is determined by which [`GLPointers`] function is used
/// within [`set_uniform`].
///
/// When implementing [`set_uniform`]:
///
/// - You should use the `glProgramUniform*` functions, not the regular `glUniform*` functions.
/// - You should not call any other [`GLPointers`] functions, just the `glProgramUniform{1234}{idf ui}[v]` family of
///   functions.
/// - The function you do use **must** be able to support more than one value. That is, **it should be one of the `v`
///   variants** of `glProgramUniform*`. This is because there exists a blanket implementation of this trait that covers
///   any slice of uniforms, which relies on the assumption that it is sound to call `T::set_uniform` with an
///   arbitrarily sized buffer along with a different `count` parameter.
///
/// An implementation of this trait is perhaps best given by an example.
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
///         // There are three floats, but that's only 1 of the things that `glProgramUniform3fv` is expecting:
///         1
///     }
///
///     unsafe fn set_uniform(
///         gl: &GLPointers,
///         program: GLuint,
///         location: GLint,
///         count: GLsizei,
///         value: *const Self::PtrType
///     ) {
///         // Vec3 doesn't need to do anything fancy here, so we just pass everything through directly. This method now
///         // basically serves to forward `set_uniform` to the correct `platform_uniform_*` function call.
///         gl.platform_uniform_3fv(program, location, count, value)
///     }
/// }
/// ```
///
/// The reason that `set_uniform` is a trait method is so that it can wrap a function that provides a bit more context
/// to the underlying [`GLPointers`] call. For example, matrices need more than a `count` and a `value` pointer:
///
/// ```
/// unsafe impl Uniform for Mat4 {
///     type PtrType = f32;
///
///     // --- snip ---
///
///     unsafe fn set_uniform(
///         gl: &GLPointers,
///         program: GLuint,
///         location: GLint,
///         count: GLsizei,
///         value: *const Self::PtrType,
///     ) {
///         gl.program_uniform_matrix_4fv(program, location, count, /* transpose: */ false as GLboolean, value)
///     }
/// }
/// ```
///
/// Now, [`Program::uniform`] can call `<Mat4 as Uniform>::set_uniform` with just a location, count, and value; without
/// needing to supply an extra parameter. Otherwise, we would need a way to encapsulate the extra parameter within the
/// trait signature, something that isn't really possible (at least in any nice way).
///
/// [`get_ptr`]: Uniform::get_ptr
/// [`count`]: Uniform::count
/// [`set_uniform`]: Uniform::set_uniform
pub unsafe trait Uniform {
    /// Which `GLtype` this value is; used for pointer casting.
    ///
    /// For example, a `vec3` uniform is would use `glProgramUniform3fv`, and so the [`Vec3`] struct implements this
    /// trait with a `PtrType` of [`GLfloat`] to go along with its call to [`GLPointers::uniform_3fv`].
    type PtrType;

    /// Converts a reference of this type into a pointer. The return value of this function is fed into
    /// [`set_uniform`][`Uniform::set_uniform`] by [`Program::uniform`].
    ///
    /// The default (just a call to [`std::ptr::from_ref`]) should suffice for most implementations. This function is
    /// available to be overridden for any types that implement their own to-pointer functionality.
    fn get_ptr(&self) -> *const Self::PtrType {
        from_ref(self).cast()
    }

    /// Determines the number of individual values this item is made up of. The return value of this function is fed
    /// into [`set_uniform`][`Uniform::set_uniform`] by [`Program::uniform`].
    ///
    /// For most implementations, this function simply returns `1`. It is only greater than 1 when an _array_ of this
    /// type value is being sent; however, there is already a blanket implementation that handles sending a slice of any
    /// `Uniform`-implemented value, so most uses need not worry about this detail.
    fn count(&self) -> GLsizei {
        1
    }

    /// Wrapper for the raw [`GLPointers`] function that sends one or more uniforms of this type to OpenGL.
    ///
    /// You should generally not call this method; use [`Program::uniform`] instead. That function guarantees that the
    /// value of `count` and `value` will always be the result of [`get_ptr()`] and [`count()`].
    ///
    /// [`get_ptr()`]: Uniform::get_ptr
    /// [`count()`]: Uniform::count
    unsafe fn set_uniform(
        gl: &GLPointers,
        program: GLuint,
        location: GLint,
        count: GLsizei,
        value: *const Self::PtrType,
    );
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

            unsafe fn set_uniform(
                gl: &GLPointers,
                program: GLuint,
                location: GLint,
                count: GLsizei,
                value: *const Self::PtrType
            ) {
                unsafe { gl.$func(program, location, count, false as GLboolean, value) } // transpose = false
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
                1 // count is 1 even for arrays (simply sending 1x of [f32; 4], for example)
            }

            unsafe fn set_uniform(
                gl: &GLPointers,
                program: GLuint,
                location: GLint,
                count: GLsizei,
                value: *const Self::PtrType
            ) {
                unsafe { gl.$func(program, location, count, value) }
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

impl_uniform!(from_ref, f32, program_uniform_1fv);
impl_uniform!(as_ptr, [f32; 1], program_uniform_1fv);
impl_uniform!(as_ptr, [f32; 2], program_uniform_2fv);
impl_uniform!(as_ptr, [f32; 3], program_uniform_3fv);
impl_uniform!(as_ptr, [f32; 4], program_uniform_4fv);

impl_uniform!(from_ref, i32, program_uniform_1iv);
impl_uniform!(as_ptr, [i32; 1], program_uniform_1iv);
impl_uniform!(as_ptr, [i32; 2], program_uniform_2iv);
impl_uniform!(as_ptr, [i32; 3], program_uniform_3iv);
impl_uniform!(as_ptr, [i32; 4], program_uniform_4iv);

impl_uniform!(from_ref, u32, program_uniform_1uiv);
impl_uniform!(as_ptr, [u32; 1], program_uniform_1uiv);
impl_uniform!(as_ptr, [u32; 2], program_uniform_2uiv);
impl_uniform!(as_ptr, [u32; 3], program_uniform_3uiv);
impl_uniform!(as_ptr, [u32; 4], program_uniform_4uiv);

impl_uniform!(from_ref, bool, program_uniform_1uiv);
impl_uniform!(as_ptr, [bool; 1], program_uniform_1uiv);
impl_uniform!(as_ptr, [bool; 2], program_uniform_2uiv);
impl_uniform!(as_ptr, [bool; 3], program_uniform_3uiv);
impl_uniform!(as_ptr, [bool; 4], program_uniform_4uiv);

impl_uniform!(as_ptr, Vec2, program_uniform_2fv);
impl_uniform!(as_ptr, Vec3, program_uniform_3fv);
impl_uniform!(as_ptr, Vec4, program_uniform_4fv);

impl_uniform!(matrix, Mat2, program_uniform_matrix_2fv);
impl_uniform!(matrix, Mat3, program_uniform_matrix_3fv);
impl_uniform!(matrix, Mat4, program_uniform_matrix_4fv);

impl_uniform!(matrix, [[f32; 2]; 2], program_uniform_matrix_2fv);
impl_uniform!(matrix, [[f32; 3]; 3], program_uniform_matrix_3fv);
impl_uniform!(matrix, [[f32; 4]; 4], program_uniform_matrix_4fv);

/// Because all uniforms (should) make use of the `Uniform*v` functions, it can be safely implemented it for all slices
/// by simply casting the pointer to the slice.
unsafe impl<T> Uniform for &[T]
where
    T: Uniform,
{
    type PtrType = T::PtrType;

    fn get_ptr(&self) -> *const Self::PtrType {
        <[T]>::as_ptr(self).cast() // reference to `[T]` -> `*const [T]` -> cast to `*const T::PtrType`
    }

    fn count(&self) -> GLsizei {
        convert!(self.len(), GLsizei, "number of uniform values")
    }

    unsafe fn set_uniform(
        gl: &GLPointers,
        program: GLuint,
        location: GLint,
        count: GLsizei,
        ptr: *const Self::PtrType,
    ) {
        // To set an array's uniform values, simply use the same wrapper function that the base type defined. The
        // overrides of `count()` and `as_ptr()` will be used for the `count` and `ptr` parameters.
        T::set_uniform(gl, program, location, count, ptr)
    }
}

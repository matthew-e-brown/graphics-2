mod uniform;

use std::ffi::c_char;
use std::rc::Rc;
use std::{mem, ptr};

pub use uniform::*;

use crate::raw::types::*;
use crate::raw::{GLPointers, COMPILE_STATUS, INFO_LOG_LENGTH, LINK_STATUS, TRUE as GL_TRUE};
use crate::{convert, gl_enum, GLContext};


gl_enum! {
    /// Types of shader objects/shader stages.
    pub enum ShaderType {
        Compute => COMPUTE_SHADER,
        Fragment => FRAGMENT_SHADER,
        Geometry => GEOMETRY_SHADER,
        TessellationControl => TESS_CONTROL_SHADER,
        TessellationEvaluation => TESS_EVALUATION_SHADER,
        Vertex => VERTEX_SHADER,
    }
}


impl GLContext {
    /// Creates a new [shader object][Shader].
    pub fn create_shader(&self, shader_type: ShaderType) -> Shader {
        let name = unsafe { self.gl.create_shader(shader_type.into_raw()) };
        let ptrs = Rc::clone(&self.gl);
        Shader {
            gl: ptrs,
            id: name,
            ty: shader_type,
            is_compiled: false,
            is_dirty: false,
        }
    }

    /// Creates a new [program object][Program].
    pub fn create_program(&self) -> Program {
        let name = unsafe { self.gl.create_program() };
        let ptrs = self.gl.clone();
        Program {
            gl: ptrs,
            id: name,
            is_linked: false,
            is_dirty: false,
            attached_shaders: Vec::with_capacity(2), // Start with space for vertex + fragment
        }
    }
}


/// An OpenGL shader object.
///
/// Dropping this type calls `glDeleteShader`.
///
/// Note that `glDeleteShader` does not delete any shaders that are currently attached to a [program object][Program].
/// They are cleaned up once detached from the program object (or once the program object is deleted).
pub struct Shader {
    /// A pointer to loaded OpenGL functions.
    pub(crate) gl: Rc<GLPointers>,
    /// The "name" of this shader. OpenGL also calls this a name.
    pub(crate) id: GLuint,
    /// The type of this shader.
    ty: ShaderType,
    /// Whether or not this shader is compiled.
    is_compiled: bool,
    /// Whether or not this shader has had its source code updated since it was last compiled.
    is_dirty: bool,
}

impl Shader {
    /// Gets this shader object's ID. OpenGL also refers to this value as the object's "name".
    pub const fn id(&self) -> GLuint {
        self.id
    }

    /// Gets this shader object's type.
    pub const fn ty(&self) -> ShaderType {
        self.ty
    }

    /// Checks whether or not this shader has been compiled already.
    pub const fn is_compiled(&self) -> bool {
        self.is_compiled
    }

    /// Checks whether or not this shader has had its source-code updated since it was last compiled.
    ///
    /// This is a separate flag than `is_compiled` because a shader object's source code can be updated after its old
    /// code has been compiled.
    pub const fn is_dirty(&self) -> bool {
        self.is_dirty
    }

    /// Set the source code for this shader object.
    ///
    /// Maps to `glShaderSource`.
    pub fn source<I, S>(&mut self, strings: I)
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let strings = strings.into_iter();
        let s_hint = strings.size_hint();
        let s_hint = s_hint.1.unwrap_or(s_hint.0).max(1);

        // We don't need to reallocate into NUL-terminated buffers, since they're passed with lengths. But we do still
        // need to create a buffer of pointers and lengths to pass to OpenGL.
        let mut str_ptrs = Vec::with_capacity(s_hint);
        let mut str_lens = Vec::with_capacity(s_hint);
        let mut count = 0usize;

        for s in strings {
            let str = s.as_ref();
            let ptr = str.as_bytes().as_ptr().cast::<c_char>();
            let len = convert!(str.len(), GLint, "shader source string length");

            str_ptrs.push(ptr);
            str_lens.push(len);
            count += 1;
        }

        let count = convert!(count, GLsizei, "number of shader source strings");
        unsafe { self.gl.shader_source(self.id, count, str_ptrs.as_ptr(), str_lens.as_ptr()) }

        self.is_dirty = true;
    }

    /// Compile this shader object.
    ///
    /// If compilation is unsuccessful, this [shader's info log][Self::get_info_log] is returned as an error.
    pub fn compile(&mut self) -> Result<(), String> {
        let success = unsafe {
            let mut result: GLint = 0;
            self.gl.compile_shader(self.id);
            self.gl.get_shader_iv(self.id, COMPILE_STATUS, &mut result);
            result == (GL_TRUE as GLint)
        };

        if success {
            self.is_compiled = true;
            self.is_dirty = false;
            Ok(())
        } else {
            let info_log = self.get_info_log().unwrap_or_else(|| "[NO SHADER INFO LOG]".to_string());
            Err(info_log)
        }
    }

    /// Get the current info log for this shader object, if any.
    pub fn get_info_log(&self) -> Option<String> {
        let mut log_size = 0;
        unsafe {
            self.gl.get_shader_iv(self.id, INFO_LOG_LENGTH, &mut log_size);
        }

        if log_size <= 0 {
            return None;
        }

        // subtract 1 since OpenGL returns an info log with a NUL byte we don't need
        let log_size = log_size - 1;
        let mut buffer = vec![0; log_size as usize];
        unsafe {
            let buf_ptr = buffer.as_mut_ptr().cast(); // ptr to write string into
            let len_ptr = ptr::null_mut(); // OpenGL also writes the final size of the string, but we already know it
            self.gl.get_shader_info_log(self.id, log_size, len_ptr, buf_ptr);
        }

        let str = String::from_utf8_lossy(&buffer[..]);
        Some(str.into())
    }

    /// Delete this shader.
    pub fn delete(self) {
        // let `self` drop.
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        unsafe { self.gl.delete_shader(self.id) }
    }
}


/// An OpenGL program object.
///
/// Dropping this type calls `glDeleteProgram`.
///
/// Note that any [shaders][Shader] currently attached to this program object will not be deleted by a call to
/// `glDeleteShader`. `glDeleteShader` will merely mark them for deletion until they are detached from this program (or
/// until this program itself is deleted).
pub struct Program {
    /// A pointer to loaded OpenGL functions.
    pub(crate) gl: Rc<GLPointers>,
    /// The "name" of this program object.
    pub(crate) id: GLuint,
    /// Whether or not this program has been linked yet.
    is_linked: bool,
    /// Whether or not this program has had its attached shader objects modified since it was last linked.
    is_dirty: bool,
    /// IDs and types of all attached [shader objects][Shader].
    attached_shaders: Vec<(GLuint, ShaderType)>,
}

impl Program {
    /// Gets this program object's ID. OpenGL also refers to this value as the object's "name".
    pub const fn id(&self) -> GLuint {
        self.id
    }

    /// Checks whether or not this shader program has been linked already.
    pub const fn is_linked(&self) -> bool {
        self.is_linked
    }

    /// Checks whether or not this shader program has had any extra shaders attached to it since it was last linked.
    ///
    /// This is a separate flag than `is_linked` because a shader program's linked state is not modified by subsequent
    /// attaches/detaches.
    pub const fn is_dirty(&self) -> bool {
        self.is_dirty
    }

    /// Attach a [shader object][Shader] to this program object.
    pub fn attach_shader(&mut self, shader: &Shader) {
        unsafe {
            self.gl.attach_shader(self.id, shader.id);
        }

        self.attached_shaders.push((shader.id, shader.ty));
        self.is_dirty = true;
    }

    /// Detach a [shader object][Shader] from this program object.
    ///
    /// Returns `true` if the given shader was attached to the program.
    pub fn detach_shader(&mut self, shader: &Shader) -> bool {
        self.detach_shader_by_id(shader.id)
    }

    /// Detaches a [shader object][Shader] from this program object, even if all you have is its ID.
    pub fn detach_shader_by_id(&mut self, shader_id: GLuint) -> bool {
        // Check if the given ID exists in our list of attached shaders.
        let pos = self.attached_shaders.iter().position(|(id, _)| *id == shader_id);
        if let Some(idx) = pos {
            // If so, remove the shader and return `true`
            unsafe {
                self.gl.detach_shader(self.id, shader_id);
            }

            self.attached_shaders.remove(idx);
            true
        } else {
            // Otherwise, do nothing and return false.
            false
        }
    }

    /// Detaches all [shader objects][Shader] from this program offset.
    pub fn detach_all_shaders(&mut self) {
        // Get the previously attached shaders.
        let attached = mem::replace(&mut self.attached_shaders, Vec::with_capacity(2));
        for (shader_id, _) in attached {
            unsafe {
                self.gl.detach_shader(self.id, shader_id);
            }
        }
    }

    /// Returns a list of all the currently attached shaders, in the form of their IDs and types.
    pub fn attached_shaders(&self) -> &[(GLuint, ShaderType)] {
        &self.attached_shaders[..]
    }

    /// Links all currently attached shader objects into one final compiled program object.
    ///
    /// If linkage fails, this program object's [info log][Self::get_info_log] is returned as an error.
    pub fn link(&mut self) -> Result<(), String> {
        let success = unsafe {
            let mut result: GLint = 0;
            self.gl.link_program(self.id);
            self.gl.get_program_iv(self.id, LINK_STATUS, &mut result);
            result == (GL_TRUE as GLint)
        };

        if success {
            self.is_linked = true;
            self.is_dirty = false;
            Ok(())
        } else {
            let info_log = self.get_info_log().unwrap_or_else(|| "[NO PROGRAM INFO LOG]".to_string());
            Err(info_log)
        }
    }

    pub fn get_info_log(&self) -> Option<String> {
        let mut log_size = 0;
        unsafe {
            self.gl.get_program_iv(self.id, INFO_LOG_LENGTH, &mut log_size);
        }

        if log_size <= 0 {
            return None;
        }

        let log_size = log_size - 1;
        let mut buffer = vec![0; log_size as usize];
        unsafe {
            let buf_ptr = buffer.as_mut_ptr().cast(); // ptr to write string into
            let len_ptr = ptr::null_mut(); // OpenGL also writes the final size of the string, but we already know it
            self.gl.get_program_info_log(self.id, log_size, len_ptr, buf_ptr);
        }

        let str = String::from_utf8_lossy(&buffer[..]);
        Some(str.into())
    }

    /// Delete this program object.
    pub fn delete(self) {
        // let `self` drop.
    }
}

impl Drop for Program {
    fn drop(&mut self) {
        unsafe { self.gl.delete_program(self.id) }
    }
}

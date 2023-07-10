use gl::types::*;

use crate::{gl_enum, ObjectCreationError};


gl_enum! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub enum ShaderType {
        Compute => COMPUTE_SHADER,
        Fragment => FRAGMENT_SHADER,
        Geometry => GEOMETRY_SHADER,
        TessellationControl => TESS_CONTROL_SHADER,
        TessellationEvaluation => TESS_EVALUATION_SHADER,
        Vertex => VERTEX_SHADER,
    }
}


/// A shader object in OpenGL.
#[derive(Debug, Hash)]
pub struct Shader {
    pub(crate) name: GLuint,
    shader_type: ShaderType,
}

impl Drop for Shader {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteShader(self.name);
        }
    }
}

impl Shader {
    /// Returns this shader's type.
    pub fn shader_type(&self) -> ShaderType {
        self.shader_type
    }

    /// Creates a new shader.
    ///
    /// This function wraps [`glCreateShader`].
    ///
    /// [`glCreateShader`]: https://registry.khronos.org/OpenGL-Refpages/gl4/html/glCreateShader.xhtml
    pub fn new(shader_type: ShaderType) -> Result<Self, ObjectCreationError> {
        // SAFETY: Only other possible error is if `shaderType` is not a valid value, which our enum guarantees (see
        // reference).
        let name = unsafe { gl::CreateShader(shader_type.into()) };
        if name != 0 {
            Ok(Self { name, shader_type })
        } else {
            Err(ObjectCreationError("shader"))
        }
    }

    /// Sets (replaces) the source code for this shader object.
    ///
    /// This function panics if:
    ///
    /// - `strings` has a length that does not fit into a [`GLsizei`] (usually [`i32`]), or
    /// - any of the strings in `strings` has a length that does not fit into a [`GLint`] (usually [`i32`]).
    ///
    /// This function wraps [`glShaderSource`].
    ///
    /// [`glShaderSource`]: https://registry.khronos.org/OpenGL-Refpages/gl4/html/glShaderSource.xhtml
    pub fn set_source<I, S>(&mut self, strings: I)
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let mut count = 0;
        let mut ptrs = Vec::with_capacity(count);
        let mut lens = Vec::with_capacity(count);

        for (i, string) in strings.into_iter().enumerate() {
            let string = string.as_ref();

            count += 1;
            ptrs[i] = string.as_bytes().as_ptr().cast();
            lens[i] = string
                .len()
                .try_into()
                .expect("shader source string size should be less than `GLint::MAX`");
        }

        let count = count
            .try_into()
            .expect("fewer than `GLsizei::MAX` shader source strings should be provided");

        // SAFETY:
        // - GL_INVALID_VALUE is generated if `shader` is not a valid shader -- covered by constructor
        // - GL_INVALID_OPERATION is generated if `count` is less than zero -- handled by `usize` never being negative
        unsafe {
            gl::ShaderSource(self.name, count, ptrs.as_ptr(), lens.as_ptr());
        }
    }

    /// Compiles this shader object using the source code string(s) that have been copied into it with
    /// [`set_source`][Self::set_source].
    ///
    /// If unsuccessful, the shader's info log is returned from [`get_info_log`][Self::get_info_log].
    ///
    /// This function wraps calls to [`glGetShader`] and [`glCompileShader`].
    ///
    /// [`glGetShader`]: https://registry.khronos.org/OpenGL-Refpages/gl4/html/glGetShader.xhtml
    /// [`glCompileShader`]: https://registry.khronos.org/OpenGL-Refpages/gl4/html/glCompileShader.xhtml
    pub fn compile(&mut self) -> Result<(), String> {
        unsafe {
            gl::CompileShader(self.name);
        }

        let success = unsafe {
            let mut status = 0;
            gl::GetShaderiv(self.name, gl::COMPILE_STATUS, &mut status);
            status as GLboolean == gl::TRUE
        };

        if success {
            Ok(())
        } else {
            let info_log = self.get_log_info().unwrap_or_else(|| "[NO SHADER INFO LOG]".to_string());
            Err(info_log)
        }
    }

    /// Gets any log information from the previous shader compilation attempt.
    ///
    /// This function wraps calls to both [`glGetShader`] (to get the buffer size to allocate for the `String`) and
    /// [`glGetShaderInfoLog`].
    ///
    /// [`glGetShader`]: https://registry.khronos.org/OpenGL-Refpages/gl4/html/glGetShader.xhtml
    /// [`glGetShaderInfoLog`]: https://registry.khronos.org/OpenGL-Refpages/gl4/html/glGetShaderInfoLog.xhtml
    pub fn get_log_info(&self) -> Option<String> {
        let mut log_size = 0;
        unsafe {
            gl::GetShaderiv(self.name, gl::INFO_LOG_LENGTH, &mut log_size);
        }

        if log_size <= 0 {
            return None;
        }

        // -1 because we don't need the NULL terminator at the end
        let mut buffer = vec![0; log_size as usize - 1];
        unsafe {
            gl::GetShaderInfoLog(self.name, log_size - 1, std::ptr::null_mut(), buffer.as_mut_ptr().cast());
        }

        Some(String::from_utf8_lossy(&buffer).into())
    }
}


/// An OpenGL program, made up multiple linked [shaders][Shader].
#[derive(Debug, Hash)]
pub struct Program {
    pub(crate) name: GLuint,
}

impl Drop for Program {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteProgram(self.name);
        }
    }
}

// TODO: add documentation for program

impl Program {
    pub fn new() -> Result<Self, ObjectCreationError> {
        let name = unsafe { gl::CreateProgram() };
        if name != 0 {
            Ok(Self { name })
        } else {
            Err(ObjectCreationError("a program"))
        }
    }

    pub fn attach_shader(&mut self, shader: &Shader) {
        // SAFETY: The only errors listed by the reference for this function are when the parameters are invalid.
        unsafe {
            gl::AttachShader(self.name, shader.name);
        }
    }

    pub fn detach_shader(&mut self, shader: &Shader) {
        // SAFETY: The only errors listed by the reference at for invalid arguments, or when the shader is not
        // attached.
        unsafe {
            gl::DetachShader(self.name, shader.name);
        }
    }

    pub fn link(&mut self) -> Result<(), String> {
        unsafe {
            gl::LinkProgram(self.name);
        }

        let success = unsafe {
            let mut status = 0;
            gl::GetProgramiv(self.name, gl::LINK_STATUS, &mut status);
            status as GLboolean == gl::TRUE
        };

        if success {
            Ok(())
        } else {
            let info_log = self.get_log_info().unwrap_or_else(|| "[NO PROGRAM INFO LOG]".to_string());
            Err(info_log)
        }
    }

    pub fn use_program(&self) {
        unsafe {
            gl::UseProgram(self.name);
        }
    }

    /// Gets any log information from the previous program linkage attempt.
    pub fn get_log_info(&self) -> Option<String> {
        let mut log_size = 0;
        unsafe {
            gl::GetProgramiv(self.name, gl::INFO_LOG_LENGTH, &mut log_size);
        }

        if log_size <= 0 {
            return None;
        }

        let mut buffer = vec![0; log_size as usize - 1];
        unsafe {
            gl::GetProgramInfoLog(self.name, log_size - 1, std::ptr::null_mut(), buffer.as_mut_ptr().cast());
        }

        Some(String::from_utf8_lossy(&buffer).into())
    }
}

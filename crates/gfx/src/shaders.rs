use std::fmt::Display;

use gl::types::*;


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ShaderType {
    Compute,
    Fragment,
    Geometry,
    TessellationControl,
    TessellationEvaluation,
    Vertex,
}

impl From<ShaderType> for GLenum {
    fn from(value: ShaderType) -> Self {
        match value {
            ShaderType::Compute => gl::COMPUTE_SHADER,
            ShaderType::Fragment => gl::FRAGMENT_SHADER,
            ShaderType::Geometry => gl::GEOMETRY_SHADER,
            ShaderType::TessellationControl => gl::TESS_CONTROL_SHADER,
            ShaderType::TessellationEvaluation => gl::TESS_EVALUATION_SHADER,
            ShaderType::Vertex => gl::VERTEX_SHADER,
        }
    }
}


impl Display for ShaderType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str(match self {
            ShaderType::Compute => "compute",
            ShaderType::Fragment => "fragment",
            ShaderType::Geometry => "geometry",
            ShaderType::TessellationControl => "tessellation control",
            ShaderType::TessellationEvaluation => "tessellation evaluation",
            ShaderType::Vertex => "vertex",
        })
    }
}


/// A shader object in OpenGL.
#[derive(Debug)]
pub struct Shader {
    name: GLuint,
    ty: ShaderType,
}

impl Drop for Shader {
    fn drop(&mut self) {
        unsafe { gl::DeleteShader(self.name) };
    }
}

impl Shader {
    /// The _"name"_ that OpenGL uses for this shader under the hood.
    pub fn gl_name(&self) -> GLuint {
        self.name
    }

    /// Gets any log information from the previous shader compilation attempt.
    pub fn get_log_info(&self) -> Option<String> {
        let mut log_size = 0;
        unsafe { gl::GetShaderiv(self.name, gl::INFO_LOG_LENGTH, &mut log_size) };

        if log_size <= 0 {
            return None;
        }

        // -1 because we don't need the NULL at the end
        // (https://registry.khronos.org/OpenGL-Refpages/es2.0/xhtml/glGetShaderiv.xml)
        let mut buffer = vec![0; log_size as usize - 1];
        unsafe { gl::GetShaderInfoLog(self.name, log_size - 1, std::ptr::null_mut(), buffer.as_mut_ptr().cast()) };

        Some(String::from_utf8_lossy(&buffer).into())
    }

    /// Compiles the given source string into a new shader object.
    ///
    /// If compilation is unsuccessful, the shader's error log is returned.
    pub fn compile(ty: ShaderType, src: &str) -> Result<Self, String> {
        let name = unsafe { gl::CreateShader(ty.into()) };
        let shader = Self { name, ty };

        // Remove potentially breaking whitespace from start, before `#version`
        let src = src.trim_start();
        let src_ptr = src.as_bytes().as_ptr().cast();
        let src_len = src
            .as_bytes()
            .len()
            .try_into()
            .map_err(|_| "shader source is too long".to_owned())?;

        // Set the source for the shader and compile it
        unsafe {
            gl::ShaderSource(name, 1, &src_ptr, &src_len);
            gl::CompileShader(name);
        }

        // Check if the compilation was successful
        let success = unsafe {
            let mut status = 0;
            gl::GetShaderiv(name, gl::COMPILE_STATUS, &mut status);
            status as GLboolean == gl::TRUE
        };

        if success {
            Ok(shader)
        } else {
            let log_output = shader
                .get_log_info()
                .unwrap_or_else(|| "shader had no log information after failing to compile".to_owned());
            Err(log_output)
        }
    }
}


/// An OpenGL program, made up multiple linked [shaders][Shader].
#[derive(Debug)]
pub struct Program {
    name: GLuint,
}

impl Drop for Program {
    fn drop(&mut self) {
        unsafe { gl::DeleteProgram(self.name) };
    }
}

impl Program {
    /// The _"name"_ that OpenGL uses for this program under the hood.
    pub fn gl_name(&self) -> GLuint {
        self.name
    }

    /// Gets any log information from the previous program linkage attempt.
    pub fn get_log_info(&self) -> Option<String> {
        let mut log_size = 0;
        unsafe { gl::GetProgramiv(self.name, gl::INFO_LOG_LENGTH, &mut log_size) };

        if log_size <= 0 {
            return None;
        }

        let mut buffer = vec![0; log_size as usize - 1];
        unsafe { gl::GetProgramInfoLog(self.name, log_size - 1, std::ptr::null_mut(), buffer.as_mut_ptr().cast()) };

        Some(String::from_utf8_lossy(&buffer).into())
    }

    /// Creates a program by linking the given shaders.
    ///
    /// This call will fail if multiple of the same type of shader are used.
    pub fn link(shaders: &[Shader]) -> Result<Self, String> {
        // Ensure that there is at most one shader of each type. *Technically*, it is allowed to have multiple shaders
        // of the same stage, but the Khronos wiki recommends against ever doing this.
        // https://www.khronos.org/opengl/wiki/Shader_Compilation#Program_setup
        for shader in shaders {
            for other in shaders {
                // If this is a different object and it has the same type
                if shader.name != other.name && shader.ty == other.ty {
                    let message = format!("attempted to create program with multiple {} shaders", shader.ty);
                    return Err(message);
                }
            }
        }

        // Create our program
        let name = unsafe { gl::CreateProgram() };
        let program = Self { name };

        // Attach all the shaders
        for shader in shaders {
            unsafe { gl::AttachShader(program.name, shader.name) };
        }

        // Then link them all into one big compiled binary
        unsafe { gl::LinkProgram(program.name) };

        // The wiki says that, now that they're all linked into one binary, we should detach the shader objects (whether
        // or not we are going to delete them)
        for shader in shaders {
            unsafe { gl::DetachShader(program.name, shader.name) }
        }

        // Now error check
        let success = unsafe {
            let mut status = 0;
            gl::GetProgramiv(name, gl::LINK_STATUS, &mut status);
            status as GLboolean == gl::TRUE
        };

        if success {
            Ok(program)
        } else {
            let log_output = program
                .get_log_info()
                .unwrap_or_else(|| "program had no log information after failing to link".to_owned());
            Err(log_output)
        }
    }
}

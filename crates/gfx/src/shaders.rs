use std::fmt::Display;

use gl::types::*;


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ShaderType {
    Vertex,
    Fragment,
    Geometry,
    Compute,
    TessellationControl,
    TessellationEvaluation,
}

impl ShaderType {
    pub const fn gl_enum(&self) -> GLenum {
        match self {
            ShaderType::Vertex => gl::VERTEX_SHADER,
            ShaderType::Fragment => gl::FRAGMENT_SHADER,
            ShaderType::Geometry => gl::GEOMETRY_SHADER,
            ShaderType::Compute => gl::COMPUTE_SHADER,
            ShaderType::TessellationControl => gl::TESS_CONTROL_SHADER,
            ShaderType::TessellationEvaluation => gl::TESS_EVALUATION_SHADER,
        }
    }
}

impl Display for ShaderType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            ShaderType::Vertex => "vertex",
            ShaderType::Fragment => "fragment",
            ShaderType::Geometry => "geometry",
            ShaderType::Compute => "compute",
            ShaderType::TessellationControl => "tessellation control",
            ShaderType::TessellationEvaluation => "tessellation evaluation",
        })
    }
}


#[derive(Debug)]
pub struct ShaderObject {
    id: GLuint,
    ty: ShaderType,
}

impl Drop for ShaderObject {
    fn drop(&mut self) {
        unsafe { gl::DeleteShader(self.id) };
    }
}

impl ShaderObject {
    pub fn gl_id(&self) -> GLuint {
        self.id
    }

    pub fn get_log_info(&self) -> Option<String> {
        let mut log_size = 0;
        unsafe { gl::GetShaderiv(self.id, gl::INFO_LOG_LENGTH, &mut log_size) };

        if log_size <= 0 {
            return None;
        }

        // -1 because we don't need the NULL at the end
        // (https://registry.khronos.org/OpenGL-Refpages/es2.0/xhtml/glGetShaderiv.xml)
        let mut buffer = vec![0; log_size as usize - 1];
        unsafe { gl::GetShaderInfoLog(self.id, log_size - 1, std::ptr::null_mut(), buffer.as_mut_ptr().cast()) };

        Some(String::from_utf8_lossy(&buffer).into())
    }

    pub fn compile(ty: ShaderType, src: &str) -> Result<Self, String> {
        let id = unsafe { gl::CreateShader(ty.gl_enum()) };
        let shader = Self { id, ty };

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
            gl::ShaderSource(id, 1, &src_ptr, &src_len);
            gl::CompileShader(id);
        }

        // Check if the compilation was successful
        let success = unsafe {
            let mut status = 0;
            gl::GetShaderiv(id, gl::COMPILE_STATUS, &mut status);
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


#[derive(Debug)]
pub struct Program {
    id: GLuint,
}

impl Drop for Program {
    fn drop(&mut self) {
        unsafe { gl::DeleteProgram(self.id) };
    }
}

impl Program {
    pub fn gl_id(&self) -> GLuint {
        self.id
    }

    pub fn get_log_info(&self) -> Option<String> {
        let mut log_size = 0;
        unsafe { gl::GetProgramiv(self.id, gl::INFO_LOG_LENGTH, &mut log_size) };

        if log_size <= 0 {
            return None;
        }

        let mut buffer = vec![0; log_size as usize - 1];
        unsafe { gl::GetProgramInfoLog(self.id, log_size - 1, std::ptr::null_mut(), buffer.as_mut_ptr().cast()) };

        Some(String::from_utf8_lossy(&buffer).into())
    }

    pub fn link(shaders: &[ShaderObject]) -> Result<Self, String> {
        // Ensure that there is at most one shader of each type. *Technically*, it is allowed to have multiple shaders
        // of the same stage, but the Khronos wiki recommends against ever doing this.
        // https://www.khronos.org/opengl/wiki/Shader_Compilation#Program_setup
        for shader in shaders {
            for other in shaders {
                // If this is a different object and it has the same type
                if shader.id != other.id && shader.ty == other.ty {
                    return Err(format!("attempted to create program with multiple {} shaders", shader.ty));
                }
            }
        }

        // Create our program
        let id = unsafe { gl::CreateProgram() };
        let program = Self { id };

        // Attach all the shaders
        for shader in shaders {
            unsafe { gl::AttachShader(program.id, shader.id) };
        }

        // Then link them all into one big compiled binary
        unsafe { gl::LinkProgram(program.id) };

        // The wiki says that, now that they're all linked into one binary, we should detach the shader objects (whether
        // or not we are going to delete them)
        for shader in shaders {
            unsafe { gl::DetachShader(program.id, shader.id) }
        }

        // Now error check
        let success = unsafe {
            let mut status = 0;
            gl::GetProgramiv(id, gl::LINK_STATUS, &mut status);
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

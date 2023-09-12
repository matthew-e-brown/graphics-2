use std::ffi::CString;

use gl::types::*;

use crate::errors::{ObjectCreationError, ObjectCreationErrorKind as ErrKind};
use crate::gl_convert;
use crate::types::*;


pub fn create_shader(shader_type: ShaderType) -> Result<ShaderID, ObjectCreationError> {
    let name = unsafe { gl::CreateShader(shader_type.raw()) };
    if name == 0 {
        Err(ObjectCreationError::new(ErrKind::Shader))
    } else {
        Ok(ShaderID::new(name))
    }
}


pub fn shader_source<S: AsRef<str>>(shader: ShaderID, strings: impl IntoIterator<Item = S>) {
    let strings = strings.into_iter();
    let hint = strings.size_hint();
    let hint = hint.1.unwrap_or(hint.0).max(1);

    let mut count = 0usize;
    let mut ptrs = Vec::with_capacity(hint);
    let mut lens = Vec::with_capacity(hint);

    for s in strings {
        let str = s.as_ref();
        let ptr = str.as_bytes().as_ptr().cast();
        let len = gl_convert!(str.len(), GLint, "shader source string size");

        ptrs.push(ptr);
        lens.push(len);
        count += 1;
    }

    let count = gl_convert!(count, GLsizei, "number of shader source strings");
    unsafe {
        gl::ShaderSource(shader.raw(), count, ptrs.as_ptr(), lens.as_ptr());
    }
}


pub fn compile_shader(shader: ShaderID) -> Result<(), String> {
    let success = unsafe {
        let mut status = 0;
        gl::CompileShader(shader.raw());
        gl::GetShaderiv(shader.raw(), gl::COMPILE_STATUS, &mut status);
        status as GLboolean == gl::TRUE
    };

    if success {
        Ok(())
    } else {
        let info_log = get_shader_info_log(shader).unwrap_or_else(|| "[NO SHADER INFO LOG]".to_string());
        Err(info_log)
    }
}


pub fn get_shader_info_log(shader: ShaderID) -> Option<String> {
    let mut log_size = 0;
    unsafe {
        gl::GetShaderiv(shader.raw(), gl::INFO_LOG_LENGTH, &mut log_size);
    }

    if log_size <= 0 {
        return None;
    }

    // size - 1 since the info log includes a `NUL` byte that we don't need
    let log_size = log_size - 1;
    let mut buffer = vec![0; log_size as usize];
    unsafe {
        gl::GetShaderInfoLog(shader.raw(), log_size, std::ptr::null_mut(), buffer.as_mut_ptr().cast());
    }

    let str = String::from_utf8_lossy(&buffer);
    Some(str.into())
}


pub fn create_program() -> Result<ProgramID, ObjectCreationError> {
    let name = unsafe { gl::CreateProgram() };
    if name == 0 {
        Err(ObjectCreationError::new(ErrKind::Program))
    } else {
        Ok(ProgramID::new(name))
    }
}


pub fn attach_shader(program: ProgramID, shader: ShaderID) {
    unsafe {
        gl::AttachShader(program.raw(), shader.raw());
    }
}


pub fn detach_shader(program: ProgramID, shader: ShaderID) {
    unsafe {
        gl::DetachShader(program.raw(), shader.raw());
    }
}


pub fn delete_shader(shader: ShaderID) {
    unsafe {
        gl::DeleteShader(shader.raw());
    }
}


pub fn link_program(program: ProgramID) -> Result<(), String> {
    let success = unsafe {
        let mut status = 0;
        gl::LinkProgram(program.raw());
        gl::GetProgramiv(program.raw(), gl::LINK_STATUS, &mut status);
        status as GLboolean == gl::TRUE
    };

    if success {
        Ok(())
    } else {
        let info_log = get_program_info_log(program).unwrap_or_else(|| "[NO PROGRAM INFO LOG]".to_string());
        Err(info_log)
    }
}


pub fn get_program_info_log(program: ProgramID) -> Option<String> {
    let mut log_size = 0;
    unsafe {
        gl::GetProgramiv(program.raw(), gl::INFO_LOG_LENGTH, &mut log_size);
    }

    if log_size <= 0 {
        return None;
    }

    // size - 1 since the info log includes a `NUL` byte that we don't need
    let log_size = log_size - 1;
    let mut buffer = vec![0; log_size as usize];
    unsafe {
        gl::GetProgramInfoLog(program.raw(), log_size, std::ptr::null_mut(), buffer.as_mut_ptr().cast());
    }

    let str = String::from_utf8_lossy(&buffer);
    Some(str.into())
}


pub fn use_program(program: ProgramID) {
    unsafe {
        gl::UseProgram(program.raw());
    }
}


pub fn delete_program(program: ProgramID) {
    unsafe { gl::DeleteProgram(program.raw()) }
}


pub fn create_shader_program<S: AsRef<str>>(
    shader_type: ShaderType,
    strings: impl IntoIterator<Item = S>,
) -> Result<ProgramID, String> {
    let strings = strings.into_iter();
    let hint = strings.size_hint();
    let hint = hint.1.unwrap_or(hint.0).max(1);

    // This OpenGL function only accepts NUL-terminated strings, not strings+lengths. That means we need to convert all
    // our `&str` into `CString`s, which requires a re-allocation to add their NULs. Then, we need to create a buffer of
    // pointers to them to pass to OpenGL.
    let mut count = 0usize;
    let mut c_strings = Vec::with_capacity(hint);
    let mut str_ptrs = Vec::with_capacity(hint);

    for (i, string) in strings.enumerate() {
        let string = CString::new(string.as_ref())
            .map_err(|e| format!("shader source string #{i} contains a NUL-byte at position {}", e.nul_position()))?;
        let str_ptr = string.as_ptr();

        c_strings.push(string);
        str_ptrs.push(str_ptr);
        count += 1;
    }

    let count = gl_convert!(count, GLsizei, "number of shader source strings");
    let program = unsafe { gl::CreateShaderProgramv(shader_type.raw(), count, str_ptrs.as_ptr().cast()) };
    let program = ProgramID::new(program);

    if program.raw() == 0 {
        Err(ObjectCreationError::new(ErrKind::Program).to_string())
    } else {
        let linked = unsafe {
            let mut status = 0;
            gl::GetProgramiv(program.raw(), gl::LINK_STATUS, &mut status);
            status as GLboolean == gl::TRUE
        };

        if linked {
            Ok(program)
        } else {
            let info_log = get_program_info_log(program).unwrap_or_else(|| "[NO PROGRAM INFO LOG]".to_string());
            Err(info_log)
        }
    }
}

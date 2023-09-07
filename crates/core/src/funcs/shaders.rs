use gl::types::*;

use crate::errors::*;
use crate::types::*;


pub fn create_shader(shader_type: ShaderType) -> Result<ShaderID, ObjectCreationError> {
    let name = unsafe { gl::CreateShader(shader_type.into()) };
    if name == 0 {
        Err(ObjectCreationError::new("a shader"))
    } else {
        Ok(ShaderID::new(name, shader_type))
    }
}


pub fn shader_source<S: AsRef<str>>(shader: ShaderID, strings: impl IntoIterator<Item = S>) {
    let strings = strings.into_iter();
    let hint = strings.size_hint();
    let hint = hint.1.unwrap_or(hint.0).max(1);

    let mut count = 0usize;
    let mut ptrs = Vec::with_capacity(hint);
    let mut lens = Vec::with_capacity(hint);

    for string in strings {
        let string = string.as_ref();
        let ptr = string.as_bytes().as_ptr().cast();
        let len = string
            .len()
            .try_into()
            .expect("shader source string size should be less than `GLint::MAX`");

        count += 1;
        ptrs.push(ptr);
        lens.push(len);
    }

    let count = count
        .try_into()
        .expect("fewer than `GLsizei::MAX` shader source strings should be provided");

    unsafe {
        gl::ShaderSource(shader.name(), count, ptrs.as_ptr(), lens.as_ptr());
    }
}


pub fn compile_shader(shader: ShaderID) -> Result<(), String> {
    let success = unsafe {
        let mut status = 0;
        gl::CompileShader(shader.name());
        gl::GetShaderiv(shader.name(), gl::COMPILE_STATUS, &mut status);
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
        gl::GetShaderiv(shader.name(), gl::INFO_LOG_LENGTH, &mut log_size);
    }

    if log_size <= 0 {
        return None;
    }

    // Size minus one since the info log includes a `NUL` byte that we don't need
    let log_size = log_size - 1;
    let mut buffer = vec![0; log_size as usize];
    unsafe {
        gl::GetShaderInfoLog(shader.name(), log_size, std::ptr::null_mut(), buffer.as_mut_ptr().cast());
    }

    let str = String::from_utf8_lossy(&buffer);
    Some(str.into())
}


pub fn create_program() -> Result<ProgramID, ObjectCreationError> {
    let name = unsafe { gl::CreateProgram() };
    if name == 0 {
        Err(ObjectCreationError::new("a program"))
    } else {
        Ok(ProgramID::new(name))
    }
}


pub fn attach_shader(program: ProgramID, shader: ShaderID) {
    unsafe {
        gl::AttachShader(program.name(), shader.name());
    }
}


pub fn detach_shader(program: ProgramID, shader: ShaderID) {
    unsafe {
        gl::DetachShader(program.name(), shader.name());
    }
}


pub fn link_program(program: ProgramID) -> Result<(), String> {
    let success = unsafe {
        let mut status = 0;
        gl::LinkProgram(program.name());
        gl::GetProgramiv(program.name(), gl::LINK_STATUS, &mut status);
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
        gl::GetProgramiv(program.name(), gl::INFO_LOG_LENGTH, &mut log_size);
    }

    if log_size <= 0 {
        return None;
    }

    // Size minus one since the info log includes a `NUL` byte that we don't need
    let log_size = log_size - 1;
    let mut buffer = vec![0; log_size as usize];
    unsafe {
        gl::GetProgramInfoLog(program.name(), log_size, std::ptr::null_mut(), buffer.as_mut_ptr().cast());
    }

    let str = String::from_utf8_lossy(&buffer);
    Some(str.into())
}


pub fn use_program(program: ProgramID) {
    unsafe {
        gl::UseProgram(program.name());
    }
}

use std::ffi::CString;

use crate::raw::types::*;
use crate::raw::{COMPILE_STATUS, INFO_LOG_LENGTH, LINK_STATUS};
use crate::types::*;
use crate::{convert, GLContext};

impl GLContext {
    pub fn create_shader(&self, shader_type: ShaderType) -> ShaderID {
        let id = unsafe { self.gl.create_shader(shader_type.into_raw()) };
        unsafe { ShaderID::from_raw_unchecked(id) }
    }


    pub fn shader_source<I, S>(&self, shader: ShaderID, strings: I)
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let strings = strings.into_iter();
        let hint = strings.size_hint();
        let hint = hint.1.unwrap_or(hint.0).max(1);

        let mut count = 0usize;
        let mut ptrs = Vec::with_capacity(hint);
        let mut lens = Vec::with_capacity(hint);

        for s in strings {
            let str = s.as_ref();
            let ptr = str.as_bytes().as_ptr().cast();
            let len = convert!(str.len(), GLint, "shader source string size");

            ptrs.push(ptr);
            lens.push(len);
            count += 1;
        }

        let count = convert!(count, GLsizei, "number of shader source strings");
        unsafe { self.gl.shader_source(shader.into_raw(), count, ptrs.as_ptr(), lens.as_ptr()) }
    }


    pub fn compile_shader(&self, shader: ShaderID) -> Result<(), String> {
        let success = unsafe {
            let mut result = 0;
            self.gl.compile_shader(shader.into_raw());
            self.gl.get_shader_iv(shader.into_raw(), COMPILE_STATUS, &mut result);
            result != 0
        };

        if success {
            Ok(())
        } else {
            let info_log = self
                .get_shader_info_log(shader)
                .unwrap_or_else(|| "[NO SHADER INFO LOG]".to_string());
            Err(info_log)
        }
    }


    pub fn get_shader_info_log(&self, shader: ShaderID) -> Option<String> {
        let mut log_size = 0;
        unsafe { self.gl.get_shader_iv(shader.into_raw(), INFO_LOG_LENGTH, &mut log_size) };

        if log_size <= 0 {
            return None;
        }

        // size - 1 since the info log includes a `NUL` byte that we don't need
        let log_size = log_size - 1;
        let mut buffer = vec![0; log_size as usize];
        unsafe {
            let buf_ptr = buffer.as_mut_ptr().cast();
            let len_ptr = std::ptr::null_mut();
            self.gl.get_shader_info_log(shader.into_raw(), log_size, len_ptr, buf_ptr);
        }

        let str = String::from_utf8_lossy(&buffer);
        Some(str.into())
    }


    pub fn create_program(&self) -> ProgramID {
        let id = unsafe { self.gl.create_program() };
        unsafe { ProgramID::from_raw_unchecked(id) }
    }


    pub fn attach_shader(&self, program: ProgramID, shader: ShaderID) {
        unsafe { self.gl.attach_shader(program.into_raw(), shader.into_raw()) }
    }


    pub fn detach_shader(&self, program: ProgramID, shader: ShaderID) {
        unsafe { self.gl.detach_shader(program.into_raw(), shader.into_raw()) }
    }


    pub fn delete_shader(&self, shader: ShaderID) {
        unsafe { self.gl.delete_shader(shader.into_raw()) }
    }


    pub fn link_program(&self, program: ProgramID) -> Result<(), String> {
        let success = unsafe {
            let mut status = 0;
            self.gl.link_program(program.into_raw());
            self.gl.get_program_iv(program.into_raw(), LINK_STATUS, &mut status);
            status != 0
        };

        if success {
            Ok(())
        } else {
            let info_log = self
                .get_program_info_log(program)
                .unwrap_or_else(|| "[NO PROGRAM INFO LOG]".to_string());
            Err(info_log)
        }
    }


    pub fn get_program_info_log(&self, program: ProgramID) -> Option<String> {
        let mut log_size = 0;
        unsafe { self.gl.get_program_iv(program.into_raw(), INFO_LOG_LENGTH, &mut log_size) };

        if log_size <= 0 {
            return None;
        }

        // size - 1 since the info log includes a `NUL` byte that we don't need
        let log_size = log_size - 1;
        let mut buffer = vec![0; log_size as usize];
        unsafe {
            let buf_ptr = buffer.as_mut_ptr().cast();
            let len_ptr = std::ptr::null_mut();
            self.gl.get_program_info_log(program.into_raw(), log_size, len_ptr, buf_ptr);
        }

        let str = String::from_utf8_lossy(&buffer);
        Some(str.into())
    }


    pub fn use_program(&self, program: ProgramID) {
        unsafe { self.gl.use_program(program.into_raw()) }
    }


    pub fn delete_program(&self, program: ProgramID) {
        unsafe { self.gl.delete_program(program.into_raw()) }
    }


    pub fn create_shader_program<I, S>(&self, shader_type: ShaderType, strings: I) -> Result<ProgramID, String>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
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
                .map_err(|e| format!("shader source #{i} contains a NUL-byte at position {}", e.nul_position()))?;

            let str_ptr = string.as_ptr();
            c_strings.push(string);
            str_ptrs.push(str_ptr);
            count += 1;
        }

        let count = convert!(count, GLsizei, "number of shader source strings");
        let str_ptrs = str_ptrs.as_ptr().cast();

        let program = unsafe { self.gl.create_shader_program_v(shader_type.into_raw(), count, str_ptrs) };
        let program = unsafe { ProgramID::from_raw_unchecked(program) };

        let success = unsafe {
            let mut status = 0;
            self.gl.get_program_iv(program.into_raw(), LINK_STATUS, &mut status);
            status != 0
        };

        if success {
            Ok(program)
        } else {
            let info_log = self
                .get_program_info_log(program)
                .unwrap_or_else(|| "[NO PROGRAM INFO LOG]".to_string());
            Err(info_log)
        }
    }
}

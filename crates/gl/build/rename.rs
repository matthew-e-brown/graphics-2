use std::borrow::Cow;

use convert_case::{Case, Casing};


/// Converts an identity from one casing to another.
pub fn convert_ident(ident: &str, from_case: Case, to_case: Case) -> Cow<'_, str> {
    if ident.is_case(to_case) {
        Cow::Borrowed(ident)
    } else {
        Cow::Owned(ident.from_case(from_case).to_case(to_case))
    }
}


/// Converts a typename from how it appears in the raw OpenGL spec into one applicable for usage by this crate.
pub fn gl_type_to_rs(gl_typename: &str) -> Option<&'static str> {
    // cspell:disable
    #[rustfmt::skip]
    return match gl_typename {
        // Common types from OpenGL 1.1
        "GLenum"            => Some("GLEnum"),                      // super::__gl_imports::raw::c_uint;
        "GLboolean"         => Some("bool"),                        // super::__gl_imports::raw::c_uchar;
        "GLbitfield"        => Some("GLBitfield"),                  // super::__gl_imports::raw::c_uint;
        "GLvoid"            => Some("core::ffi::c_void"),           // super::__gl_imports::raw::c_void;
        "GLbyte"            => Some("i8"),                          // super::__gl_imports::raw::c_char;
        "GLshort"           => Some("i16"),                         // super::__gl_imports::raw::c_short;
        "GLint"             => Some("i32"),                         // super::__gl_imports::raw::c_int;
        "GLclampx"          => Some("i32"),                         // super::__gl_imports::raw::c_int;
        "GLubyte"           => Some("u8"),                          // super::__gl_imports::raw::c_uchar;
        "GLushort"          => Some("u16"),                         // super::__gl_imports::raw::c_ushort;
        "GLuint"            => Some("u32"),                         // super::__gl_imports::raw::c_uint;
        "GLsizei"           => Some("i32"),                         // super::__gl_imports::raw::c_int;
        "GLfloat"           => Some("f32"),                         // super::__gl_imports::raw::c_float;
        "GLclampf"          => Some("f32"),                         // super::__gl_imports::raw::c_float;
        "GLdouble"          => Some("f64"),                         // super::__gl_imports::raw::c_double;
        "GLclampd"          => Some("f64"),                         // super::__gl_imports::raw::c_double;
        "GLeglImageOES"     => Some("*const core::ffi::c_void"),    // *const super::__gl_imports::raw::c_void;
        "GLchar"            => Some("i8"),                          // super::__gl_imports::raw::c_char;
        "GLcharARB"         => Some("i8"),                          // super::__gl_imports::raw::c_char;
        // -----------------------------------------------------------------------------------------
        #[cfg(target_os = "macos")]      "GLhandleARB" => Some("*const core::ffi::c_void"), // *const super::__gl_imports::raw::c_void;
        #[cfg(not(target_os = "macos"))] "GLhandleARB" => Some("u32"),                      // super::__gl_imports::raw::c_uint;
        "GLhalfARB"         => Some("u16"),                         // super::__gl_imports::raw::c_ushort;
        "GLhalf"            => Some("u16"),                         // super::__gl_imports::raw::c_ushort;
        "GLfixed"           => Some("i32"),                         // GLint; (Must be 32 bits)
        "GLintptr"          => Some("isize"),                       // isize;
        "GLsizeiptr"        => Some("isize"),                       // isize;
        "GLint64"           => Some("i64"),                         // i64;
        "GLuint64"          => Some("u64"),                         // u64;
        "GLintptrARB"       => Some("isize"),                       // isize;
        "GLsizeiptrARB"     => Some("isize"),                       // isize;
        "GLint64EXT"        => Some("i64"),                         // i64;
        "GLuint64EXT"       => Some("u64"),                         // u64;
        "GLsync"            => Some("*const types::GLSync"),        // *const __GLsync; (with `pub enum GLSync {}` above it)
        // Vendor extension types
        "GLhalfNV"          => Some("u16"),                         // super::__gl_imports::raw::c_ushort;
        "GLvdpauSurfaceNV"  => Some("isize"),                       // GLintptr;
        // -----------------------------------------------------------------------------------------
        "GLDEBUGPROC"       => Some("types::GLDebugProc"),
        "GLDEBUGPROCARB"    => Some("types::GLDebugProc"),
        "GLDEBUGPROCKHR"    => Some("types::GLDebugProc"),
        "GLDEBUGPROCAMD"    => Some("types::GLDebugProc_AMD"),
        _ => None,
    };
    // cspell:enable
}


/// Converts a typename from how it appears after being parsed by [`gl_generator`] into one for usage by this crate.
pub fn lib_type_to_rs(lib_typename: &str) -> Cow<'_, str> {
    if lib_typename == "()" {
        return lib_typename.into();
    }

    let mut res = String::new();
    let mut str = lib_typename;

    /// Trims the given pattern off the start of the given slice, shrinking it down in the process.
    ///
    /// Returns the matched pattern for convenience.
    fn trim_start_mut<'a, 'p>(str: &mut &'a str, pat: &'p str) -> Option<&'p str> {
        if str.starts_with(pat) {
            *str = &str[pat.len()..];
            Some(pat)
        } else {
            None
        }
    }

    // Trim off any pointer types and add to our own string
    loop {
        let Some(ptr) = trim_start_mut(&mut str, "*const").or_else(|| trim_start_mut(&mut str, "*mut")) else {
            // Didn't start with either one; done.
            break;
        };

        str = str.trim_start(); // Remove extra space after *const/*mut
        res.push_str(ptr);
        res.push(' ');
    }

    // Map aliases to our types
    if let Some(_) = trim_start_mut(&mut str, "types::") {
        let our_ty = gl_type_to_rs(str).unwrap_or_else(|| panic!("unknown typename: {lib_typename}"));
        res.push_str(our_ty);
    } else if let Some(_) = trim_start_mut(&mut str, "__gl_imports::") {
        if let Some(_) = trim_start_mut(&mut str, "raw::c_void") {
            res.push_str("core::ffi::c_void");
        } else {
            unimplemented!("unknown typename: {lib_typename}");
        }
    } else {
        unimplemented!("unknown typename: {lib_typename}");
    }

    res.into()
}

use std::collections::HashMap;
use std::io::{self, Write};

use gl_generator::{Generator, Registry};
use indoc::indoc;


pub struct StructGenerator;

impl Generator for StructGenerator {
    fn write<W: Write>(&self, registry: &Registry, dest: &mut W) -> io::Result<()> {
        // `types` module:
        writeln!(dest, "pub mod types {{")?;
        write_type_aliases(dest)?;
        write_custom_types(registry, dest)?;
        writeln!(dest, "}}")?;

        Ok(())
    }
}


fn get_typename(gl_type: &str) -> &'static str {
    match gl_type {
        // Common types from OpenGL 1.1
        "GLenum" => "GLEnum",                          // super::__gl_imports::raw::c_uint;
        "GLboolean" => "bool",                         // super::__gl_imports::raw::c_uchar;
        "GLbitfield" => "GLBitfield",                  // super::__gl_imports::raw::c_uint;
        "GLvoid" => "core::ffi::c_void",               // super::__gl_imports::raw::c_void;
        "GLbyte" => "i8",                              // super::__gl_imports::raw::c_char;
        "GLshort" => "i16",                            // super::__gl_imports::raw::c_short;
        "GLint" => "i32",                              // super::__gl_imports::raw::c_int;
        "GLclampx" => "i32",                           // super::__gl_imports::raw::c_int;
        "GLubyte" => "u8",                             // super::__gl_imports::raw::c_uchar;
        "GLushort" => "u16",                           // super::__gl_imports::raw::c_ushort;
        "GLuint" => "u32",                             // super::__gl_imports::raw::c_uint;
        "GLsizei" => "i32",                            // super::__gl_imports::raw::c_int;
        "GLfloat" => "f32",                            // super::__gl_imports::raw::c_float;
        "GLclampf" => "f32",                           // super::__gl_imports::raw::c_float;
        "GLdouble" => "f64",                           // super::__gl_imports::raw::c_double;
        "GLclampd" => "f64",                           // super::__gl_imports::raw::c_double;
        "GLeglImageOES" => "*const core::ffi::c_void", // *const super::__gl_imports::raw::c_void;
        "GLchar" => "i8",                              // super::__gl_imports::raw::c_char;
        "GLcharARB" => "i8",                           // super::__gl_imports::raw::c_char;
        // -----------------------------------------------------------------------------------------
        #[cfg(target_os = "macos")]
        "GLhandleARB" => "*const core::ffi::c_void", // *const super::__gl_imports::raw::c_void;
        #[cfg(not(target_os = "macos"))]
        "GLhandleARB" => "u32", // super::__gl_imports::raw::c_uint;
        "GLhalfARB" => "u16",        // super::__gl_imports::raw::c_ushort;
        "GLhalf" => "u16",           // super::__gl_imports::raw::c_ushort;
        "GLfixed" => "i32",          // GLint; (Must be 32 bits)
        "GLintptr" => "isize",       // isize;
        "GLsizeiptr" => "isize",     // isize;
        "GLint64" => "i64",          // i64;
        "GLuint64" => "u64",         // u64;
        "GLintptrARB" => "isize",    // isize;
        "GLsizeiptrARB" => "isize",  // isize;
        "GLint64EXT" => "i64",       // i64;
        "GLuint64EXT" => "u64",      // u64;
        "GLsync" => "*const GLSync", // *const __GLsync; (with `pub enum GLSync {}` above it)
        // Vendor extension types
        "GLhalfNV" => "u16",           // super::__gl_imports::raw::c_ushort;
        "GLvdpauSurfaceNV" => "isize", // GLintptr;
        // -----------------------------------------------------------------------------------------
        _ => panic!("unknown GLtype: {gl_type}"),
    }
}


fn write_type_aliases(dest: &mut impl Write) -> io::Result<()> {
    #[rustfmt::skip]
    return dest.write_all(indoc! {r#"
        // Function pointers, used for callbacks.
        pub type GLDebugProc = Option<extern "system" fn(
            source: GLEnum,
            gl_type: GLEnum,
            id: u32,
            severity: GLEnum,
            length: i32,
            message: *const i8,
            user_param: *mut core::ffi::c_void,
        )>;

        // Opaque types, pointed to by some function parameters.

        /// Opaque type.
        pub enum GLSync {}

        /// Compatible with OpenCL `cl_context`.
        pub enum CLContext {}

        /// Compatible with OpenCL `cl_event`.
        pub enum CLEvent {}

        // Vendor extension types
        #[allow(non_camel_case_types)]
        pub type GLDebugProc_AMD = Option<extern "system" fn(
            id: u32,
            category: GLEnum,
            severity: GLEnum,
            length: i32,
            message: *const i8,
            user_param: *mut core::ffi::c_void,
        )>;

    "#}.as_bytes());
}


fn write_custom_types(registry: &Registry, dest: &mut impl Write) -> io::Result<()> {
    // FIXME: Bring back full implementations for GLEnum and GLBitfields once groups are fully supported.
    // https://github.com/matthew-e-brown/graphics-2/blob/98b6f4e7574e2d7ac62eb9190ccecb667ac8def4/crates/core/src/types/macros.rs

    #[rustfmt::skip]
    dest.write_all(indoc! {r#"
        #[repr(transparent)]
        #[derive(Clone, Copy, PartialEq, Eq, Hash)]
        #[cfg_attr(not(debug_assertions), derive(Debug))]
        pub struct GLEnum(pub(super) u32);

        // No operations defined on GLEnums; they are hard-coded values.

        impl GLEnum {
            /// Returns the underlying value of this enum.
            pub const fn into_raw(self) -> u32 {
                self.0
            }

            /// Wraps a raw `u32` as a GLEnum.
            ///
            /// # Safety
            ///
            /// This function performs no checks as to whether or not the provided value is a valid `GLEnum` value.
            pub const unsafe fn from_raw(value: u32) -> Self {
                Self(value)
            }
        }

        #[repr(transparent)]
        #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
        pub struct GLBitfield(u32);

        // GLBitfields get bitwise operations defined on them.

        impl GLBitfield {
            /// Returns the underlying value of this bitfield.
            pub const fn into_raw(self) -> u32 {
                self.0
            }

            /// Wraps a raw `u32` as a GLBitfield.
            ///
            /// # Safety
            ///
            /// This function performs no checks as to whether or not the provided value is a valid `GLBitfield` value.
            pub const unsafe fn from_raw(value: u32) -> Self {
                Self(value)
            }
        }

        // !
        impl core::ops::Not for GLBitfield { type Output = Self; fn not(self) -> Self::Output { Self(!self.0) } }

        impl core::ops::Not for &'_ GLBitfield { type Output = GLBitfield; fn not(self) -> Self::Output { !*self } }
        impl core::ops::Not for &'_ mut GLBitfield { type Output = GLBitfield; fn not(self) -> Self::Output { !*self } }

        // |, &, ^
        impl core::ops::BitOr for GLBitfield { type Output = Self; fn bitor(self, rhs: Self) -> Self::Output { GLBitfield(self.0 | rhs.0) } }
        impl core::ops::BitAnd for GLBitfield { type Output = Self; fn bitand(self, rhs: Self) -> Self::Output { GLBitfield(self.0 & rhs.0) } }
        impl core::ops::BitXor for GLBitfield { type Output = Self; fn bitxor(self, rhs: Self) -> Self::Output { GLBitfield(self.0 ^ rhs.0) } }
        impl<'l> core::ops::BitOr<GLBitfield> for &'l GLBitfield { type Output = GLBitfield; fn bitor(self, rhs: GLBitfield) -> Self::Output { GLBitfield(self.0 | rhs.0) } }
        impl<'l> core::ops::BitAnd<GLBitfield> for &'l GLBitfield { type Output = GLBitfield; fn bitand(self, rhs: GLBitfield) -> Self::Output { GLBitfield(self.0 & rhs.0) } }
        impl<'l> core::ops::BitXor<GLBitfield> for &'l GLBitfield { type Output = GLBitfield; fn bitxor(self, rhs: GLBitfield) -> Self::Output { GLBitfield(self.0 ^ rhs.0) } }
        impl<'r> core::ops::BitOr<&'r GLBitfield> for GLBitfield { type Output = GLBitfield; fn bitor(self, rhs: &'r GLBitfield) -> Self::Output { GLBitfield(self.0 | rhs.0) } }
        impl<'r> core::ops::BitAnd<&'r GLBitfield> for GLBitfield { type Output = GLBitfield; fn bitand(self, rhs: &'r GLBitfield) -> Self::Output { GLBitfield(self.0 & rhs.0) } }
        impl<'r> core::ops::BitXor<&'r GLBitfield> for GLBitfield { type Output = GLBitfield; fn bitxor(self, rhs: &'r GLBitfield) -> Self::Output { GLBitfield(self.0 ^ rhs.0) } }
        impl<'l, 'r> core::ops::BitOr<&'r GLBitfield> for &'l GLBitfield { type Output = GLBitfield; fn bitor(self, rhs: &'r GLBitfield) -> Self::Output { GLBitfield(self.0 | rhs.0) } }
        impl<'l, 'r> core::ops::BitAnd<&'r GLBitfield> for &'l GLBitfield { type Output = GLBitfield; fn bitand(self, rhs: &'r GLBitfield) -> Self::Output { GLBitfield(self.0 & rhs.0) } }
        impl<'l, 'r> core::ops::BitXor<&'r GLBitfield> for &'l GLBitfield { type Output = GLBitfield; fn bitxor(self, rhs: &'r GLBitfield) -> Self::Output { GLBitfield(self.0 ^ rhs.0) } }

        // |=, &=, ^=
        impl core::ops::BitOrAssign<GLBitfield> for GLBitfield { fn bitor_assign(&mut self, rhs: Self) { self.0 |= rhs.0 } }
        impl core::ops::BitAndAssign<GLBitfield> for GLBitfield { fn bitand_assign(&mut self, rhs: Self) { self.0 &= rhs.0 } }
        impl core::ops::BitXorAssign<GLBitfield> for GLBitfield { fn bitxor_assign(&mut self, rhs: Self) { self.0 ^= rhs.0 } }
        impl core::ops::BitOrAssign<&GLBitfield> for GLBitfield { fn bitor_assign(&mut self, rhs: &GLBitfield) { self.0 |= rhs.0 } }
        impl core::ops::BitAndAssign<&GLBitfield> for GLBitfield { fn bitand_assign(&mut self, rhs: &GLBitfield) { self.0 &= rhs.0 } }
        impl core::ops::BitXorAssign<&GLBitfield> for GLBitfield { fn bitxor_assign(&mut self, rhs: &GLBitfield) { self.0 ^= rhs.0 } }

    "#}.as_bytes())?;

    // Debug implementation for GLEnum:
    // --------------------------------

    // Check for which enums share values, then collect into vec so we can sort ascending
    let mut enum_val_map = registry
        .enums
        .iter()
        .filter(|e| e.ty == "GLenum")
        .fold(HashMap::new(), |mut map, e| {
            let val = u64::from_str_radix(e.value.trim_start_matches("0x"), 16)
                .expect("GLEnum value should be valid hex number");
            let vec = map.entry(val).or_insert_with(|| Vec::new());
            vec.push("GL_".to_owned() + &e.ident);
            map
        })
        .into_iter()
        .collect::<Vec<_>>();
    enum_val_map.sort_unstable_by_key(|&(v, _)| v);

    // Write header
    #[rustfmt::skip]
    dest.write_all(indoc! {r#"
        #[cfg(debug_assertions)]
        impl std::fmt::Debug for GLEnum {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                // Use Cow so we can use &str for most, but String for unknown values
                let debug: std::borrow::Cow<'_, str> = match self.0 {
                    // Best we can really do for any values that map to multiple enums is to print all of them
    "#}.as_bytes())?;

    // Print enum variants
    for (val, idents) in enum_val_map {
        assert!(idents.len() > 0);

        dest.write_all("            ".as_bytes())?; // indentation]
        if idents.len() == 1 {
            // Writes: `0x80CA => "GL_BLEND_DST_ALPHA".into(),\n`
            dest.write_fmt(format_args!("0x{val:04X} => \"{}\".into(),\n", idents[0]))?;
        } else {
            // Writes: `0x0020 => "OneOf(GL_COMPUTE_SHADER_BIT, ..., GL_SHADER_IMAGE_ACCESS_BARRIER_BIT)".into`
            dest.write_fmt(format_args!("0x{val:04X} => \"OneOf({})\".into(),\n", idents.join(", ")))?;
        }
    }

    #[rustfmt::skip]
    dest.write_all(indoc! {r#"
                    // Any unknown values get printed in hex for further debugging
                    unknown => format!("unknown (0x{unknown:04X})").into(),
                };

                f.debug_tuple("GLEnum").field(&&debug[..]).finish()
            }
        }
    "#}.as_bytes())?;

    Ok(())
}

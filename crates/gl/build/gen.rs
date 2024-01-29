use std::borrow::Cow;
use std::collections::{BTreeMap, BTreeSet};
use std::io::{self, Write};

use convert_case::{Case, Casing};
use gl_generator::{Enum, Generator, Registry};
use indoc::indoc;


pub struct StructGenerator;

impl Generator for StructGenerator {
    fn write<W: Write>(&self, registry: &Registry, dest: &mut W) -> io::Result<()> {
        // Start by splitting up the enums into the different sets we want.
        let [enums, bitfields, special_enums] = separate_enums(registry);

        // `types` module:
        writeln!(dest, "pub mod types {{")?;
        write_type_aliases(dest)?;
        write_custom_types(&enums, dest)?;
        writeln!(dest, "}}\n")?;

        writeln!(dest, "#[allow(unused_imports)]")?;
        writeln!(dest, "use self::types::{{GLEnum, GLBitfield}};\n")?;
        write_enum_values(&[enums, bitfields, special_enums], dest)?;
        writeln!(dest)?;

        write_gl_struct(registry, dest)?;

        Ok(())
    }
}


/// Converts a typename from how it appears in the raw OpenGL spec into one applicable for usage by this crate.
fn gl_type_to_rs(gl_typename: &str) -> Option<&'static str> {
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
fn lib_type_to_rs(lib_typename: &str) -> Cow<'_, str> {
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


fn convert_ident(ident: &str, from_case: Case, to_case: Case) -> Cow<'_, str> {
    if ident.is_case(to_case) {
        Cow::Borrowed(ident)
    } else {
        Cow::Owned(ident.from_case(from_case).to_case(to_case))
    }
}


fn parse_enum_value(val: &str) -> u32 {
    let trim = val.trim_start_matches("0x");
    u32::from_str_radix(trim, 16).unwrap_or_else(|_| panic!("GLenum value '{val}' should be valid u32 in hex"))
}


/// Separates all of the OpenGL enums into separate groups:
/// - Regular `GLenum` values,
/// - Enums tagged as `bitfield`,
/// - Enums with non-`GLenum` types.
/// In that order.
fn separate_enums(registry: &Registry) -> [BTreeSet<&Enum>; 3] {
    let mut group_regular = BTreeSet::new();
    let mut group_bitfields = BTreeSet::new();

    // Split regular and bitmask enums up
    for group in registry.groups.values() {
        for member in &group.enums {
            if let Some("bitmask") = group.enums_type.as_deref() {
                group_bitfields.insert(&member[..]);
            } else {
                group_regular.insert(&member[..]);
            }
        }
    }

    let mut regular_enums = BTreeSet::new();
    let mut bitfield_enums = BTreeSet::new();
    let mut special_enums = BTreeSet::new();

    // Filter for just the ones that're `GLenum`
    for e in &registry.enums {
        if e.ty == "GLenum" {
            if group_regular.contains(&e.ident[..]) {
                regular_enums.insert(e);
            }

            if group_bitfields.contains(&e.ident[..]) {
                bitfield_enums.insert(e);
            }
        } else {
            special_enums.insert(e);
        }
    }

    [regular_enums, bitfield_enums, special_enums]
}


fn write_type_aliases(dest: &mut impl Write) -> io::Result<()> {
    #[rustfmt::skip]
    dest.write_all(indoc! {r#"
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

    "#}.as_bytes())?;
    writeln!(dest)
}


fn write_custom_types(enums: &BTreeSet<&Enum>, dest: &mut impl Write) -> io::Result<()> {
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
        pub struct GLBitfield(pub(super) u32);

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
        impl core::ops::Not for GLBitfield { type Output = Self; fn not(self) -> Self::Output { GLBitfield(!self.0) } }

        impl core::ops::Not for &'_ GLBitfield { type Output = GLBitfield; fn not(self) -> Self::Output { GLBitfield(!self.0) } }
        impl core::ops::Not for &'_ mut GLBitfield { type Output = GLBitfield; fn not(self) -> Self::Output { GLBitfield(!self.0) } }

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
    writeln!(dest)?;

    // Debug implementation for GLEnum:
    // --------------------------------

    // Get a list of all enums, but indexed val => enum directly instead of enum => data.
    let enums_by_val = enums.iter().fold(BTreeMap::new(), |mut map, e| {
        let key = parse_enum_value(&e.value);
        let val = String::from("GL_") + &convert_ident(&e.ident, Case::Snake, Case::UpperSnake);
        let vec = map.entry(key).or_insert_with(|| Vec::new());
        vec.push(val);
        map
    });

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
    for (val, idents) in enums_by_val {
        assert!(idents.len() > 0);

        write!(dest, "            ")?; // indentation
        match idents.len() {
            1 => dest.write_fmt(format_args!("0x{val:04X} => \"{}\".into(),\n", idents[0]))?,
            _ => dest.write_fmt(format_args!("0x{val:04X} => \"OneOf({})\".into(),\n", idents.join(", ")))?,
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


fn write_enum_values(enum_sets: &[BTreeSet<&Enum>; 3], dest: &mut impl Write) -> io::Result<()> {
    let [enums, bitfields, special_enums] = enum_sets;

    // Write enums
    for &e in enums {
        let ident = convert_ident(&e.ident, Case::Snake, Case::UpperSnake);
        let value = &e.value;
        dest.write_fmt(format_args!("pub const {ident}: GLEnum = GLEnum({value});\n"))?;
    }

    writeln!(dest)?;

    // Write bitmasks
    for &e in bitfields {
        let ident = convert_ident(&e.ident, Case::Snake, Case::UpperSnake);
        let value = &e.value;
        dest.write_fmt(format_args!("pub const {ident}: GLBitfield = GLBitfield({value});\n"))?;
    }

    writeln!(dest)?;

    // Write special values (except the booleans, we don't need those)
    for &e in special_enums.iter().filter(|e| e.ty != "GLboolean") {
        let ident = convert_ident(&e.ident, Case::Snake, Case::UpperSnake);
        let value = &e.value;
        let ty = gl_type_to_rs(&e.ty).unwrap_or_else(|| panic!("enum type {} should map to Rust type", e.ty));
        dest.write_fmt(format_args!("pub const {ident}: {ty} = {value};\n"))?;
    }

    Ok(())
}


fn make_fn_ptr<'a, S: AsRef<str>>(params: impl IntoIterator<Item = (S, S)>, ret_ty: &str) -> String {
    let mut result = String::from("extern \"system\" fn(");

    let params = params
        .into_iter()
        .map(|(ident, ty)| format!("{}: {}", ident.as_ref(), ty.as_ref()))
        .collect::<Vec<_>>()
        .join(", ");

    result += &params;
    result += ")";

    if ret_ty != "()" {
        result += " -> ";
        result += ret_ty;
    }

    result
}


fn write_gl_struct(registry: &Registry, dest: &mut impl Write) -> io::Result<()> {
    // Create the OpenGL context struct
    #[rustfmt::skip]
    dest.write_all(indoc! {r#"
        /// An abstraction over an OpenGL context.
        ///
        /// This struct _not really_ an "OpenGL context;" really, it is a collection of loaded function pointers for use
        /// in the current thread.
        pub struct GLContext {
    "#}.as_bytes())?;

    // println!("{:#?}", registry.cmds);
    for cmd in &registry.cmds {
        let ident = convert_ident(&cmd.proto.ident, Case::Camel, Case::Snake);
        let ret_ty = lib_type_to_rs(&cmd.proto.ty);
        let params = cmd.params.iter().map(|param| {
            let ident = convert_ident(&param.ident, Case::UpperCamel, Case::Snake);
            let ty = lib_type_to_rs(&param.ty);
            (ident, ty)
        });

        let fn_ptr = make_fn_ptr(params, &ret_ty);
        writeln!(dest, "    {ident}: Option<{fn_ptr}>,")?;
    }

    writeln!(dest, "}}")?;
    Ok(())
}

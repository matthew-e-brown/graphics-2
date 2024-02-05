use std::collections::BTreeSet;
use std::io::{self, Write};

use gl_generator::Registry;
use indoc::indoc;


pub fn write_types_module(dest: &mut impl Write) -> io::Result<()> {
    #[rustfmt::skip]
    dest.write_all(indoc! {r#"
        pub mod types {
            // Opaque types, used as pointees in some other type definitions.
            // -----------------------------------------------------------------------------

            /// Opaque type. Used as a pointee.
            pub enum GLSyncHandle {}

            /// Opaque type. Compatible with OpenCL `cl_context`.
            pub enum CLContext {}

            /// Opaque type. Compatible with OpenCL `cl_event`.
            pub enum CLEvent {}

            // Standard type aliases
            // -----------------------------------------------------------------------------

            // These types are defined in table 2.2 on page 13 of the 4.6 core specification. They are specifically
            // *not* C types, but have hard-coded sizes. Don't forget that many also have different semantic meanings,
            // despite mapping to the same underlying types.

            /// Boolean.
            pub type GLboolean = u8;
            /// Signed two's complement binary integer.
            pub type GLbyte = i8;
            /// Unsigned binary integer.
            pub type GLubyte = u8;
            /// Characters making up strings.
            pub type GLchar = i8;
            /// Signed two's complement binary integer.
            pub type GLshort = i16;
            /// Unsigned binary integer.
            pub type GLushort = u16;
            /// Signed two's complement binary integer.
            pub type GLint = i32;
            /// Unsigned binary integer.
            pub type GLuint = u32;
            /// Signed two's complement 16.16 scaled integer.
            pub type GLfixed = i32;
            /// Signed two's complement binary integer.
            pub type GLint64 = i64;
            /// Unsigned binary integer.
            pub type GLuint64 = u64;
            /// Non-negative binary integer size.
            pub type GLsizei = i32; // defined as 'int' in gl.xml even though spec says "non-negative"
            /// Enumerated binary integer value.
            pub type GLenum = u32;
            /// Signed two's complement binary integer (`ptrbits`).
            pub type GLintptr = isize;
            /// Non-negative binary integer size (`ptrbits`).
            pub type GLsizeiptr = isize;
            /// Sync object handle.
            pub type GLsync = *const GLSyncHandle;
            /// Bit field.
            pub type GLbitfield = u32;
            /// Half-precision floating-point value encoding in an unsigned scalar.
            pub type GLhalf = u16;
            /// Floating-point value.
            pub type GLfloat = f32;
            /// Floating-point value clamped to [0, 1].
            pub type GLclampf = f32;
            /// Floating-point value.
            pub type GLdouble = f64;
            /// Floating-point value clamped to [0, 1].
            pub type GLclampd = f64;

            // Function pointers, used for callbacks.
            // -----------------------------------------------------------------------------

            pub type GLDebugProc = Option<extern "system" fn(
                source: GLenum,
                gl_type: GLenum,
                id: GLuint,
                severity: GLenum,
                length: GLsizei,
                message: *const GLchar,
                user_param: *mut core::ffi::c_void,
            )>;

            // Vendor extension types
            // -----------------------------------------------------------------------------

            #[allow(non_camel_case_types)]
            pub type GLDebugProc_AMD = Option<extern "system" fn(
                id: GLuint,
                category: GLenum,
                severity: GLenum,
                length: GLsizei,
                message: *const GLchar,
                user_param: *mut core::ffi::c_void,
            )>;
        }
    "#}.as_bytes())?;
    writeln!(dest)
}


pub fn write_enum_values(registry: &Registry, dest: &mut impl Write) -> io::Result<()> {
    // Sort the enums into specific groups
    let (standard, bitfield, other) = {
        let mut reg_groups = BTreeSet::new();
        let mut bit_groups = BTreeSet::new();

        // Split regular and bitmask enums up
        for group in registry.groups.values() {
            for member in &group.enums {
                match group.enums_type.as_deref() {
                    None => reg_groups.insert(member.as_str()),
                    Some("bitmask") => bit_groups.insert(member.as_str()),
                    Some(other) => unimplemented!("unknown enum type: {other}"),
                };
            }
        }

        let mut standard = BTreeSet::new();
        let mut bitfield = BTreeSet::new();
        let mut other = BTreeSet::new();

        // Filter for just the ones that're `GLenum`
        for e in &registry.enums {
            match &e.ty[..] {
                "GLenum" if reg_groups.contains(e.ident.as_str()) => standard.insert(e),
                "GLenum" if bit_groups.contains(e.ident.as_str()) => bitfield.insert(e),
                _ => other.insert(e),
            };
        }

        (standard, bitfield, other)
    };

    // Then iterate over those groups and create their values
    let standard = standard.into_iter().map(|e| ("GLenum", e));
    let bitfield = bitfield.into_iter().map(|e| ("GLbitfield", e));
    let other = other.into_iter().map(|e| (&e.ty[..], e));

    for (ty, e) in standard.chain(bitfield).chain(other) {
        let ident = e.ident.as_str(); // no need to rename, they're already in `UPPER_SNAKE` from the spec
        let value = e.value.as_str();

        // Only add the linter warning thingy for the values that need it. This *should* only be the ones with little
        // x's in them, i.e. `MAT2x3` and co.
        if ident.chars().any(|c| c.is_lowercase()) {
            write!(dest, "#[allow(non_upper_case_globals)] ")?;
        }

        writeln!(dest, "pub const {ident}: {ty} = {value};")?;
    }

    Ok(())
}

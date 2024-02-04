use std::collections::{BTreeMap, BTreeSet};
use std::io::{self, Write};

use gl_generator::{Enum, Registry};
use indoc::indoc;

use crate::rename::{rename_enum_variant, rename_xml_type};


fn parse_value(val: &str) -> u32 {
    let trim = val.trim_start_matches("0x");
    u32::from_str_radix(trim, 16).expect("GLenum value should be valid hex u32")
}


/// Enums from a [`Registry`] that have been categorized appropriately.
///
/// Note that enums may appear in more than one category.
#[derive(Debug)]
pub struct SortedEnums<'a> {
    /// All non-funky `GLenum` entries.
    standard: BTreeSet<&'a Enum>,
    /// Any enum that appears in a group labelled `"bitmask"`.
    bitfield: BTreeSet<&'a Enum>,
    /// Any enum that does not have `GLenum` as its type.
    other: BTreeSet<&'a Enum>,
}

impl<'a> SortedEnums<'a> {
    pub fn from_registry(registry: &'a Registry) -> Self {
        let mut reg_groups = BTreeSet::new();
        let mut bit_groups = BTreeSet::new();

        // Split regular and bitmask enums up
        for group in registry.groups.values() {
            for member in &group.enums {
                if let Some("bitmask") = group.enums_type.as_deref() {
                    bit_groups.insert(&member[..]);
                } else {
                    reg_groups.insert(&member[..]);
                }
            }
        }

        let mut standard = BTreeSet::new();
        let mut bitfield = BTreeSet::new();
        let mut other = BTreeSet::new();

        // Filter for just the ones that're `GLenum`
        for e in &registry.enums {
            if e.ty == "GLenum" {
                if reg_groups.contains(&e.ident[..]) {
                    standard.insert(e);
                }

                if bit_groups.contains(&e.ident[..]) {
                    bitfield.insert(e);
                }
            } else {
                other.insert(e);
            }
        }

        Self { standard, bitfield, other }
    }
}


pub fn write_type_aliases(dest: &mut impl Write) -> io::Result<()> {
    #[rustfmt::skip]
    dest.write_all(indoc! {r#"
        use core::ffi::c_void;

        // Function pointers, used for callbacks.
        // -----------------------------------------------------------------------------

        pub type GLDebugProc = Option<extern "system" fn(
            source: GLEnum,
            gl_type: GLEnum,
            id: u32,
            severity: GLEnum,
            length: i32,
            message: *const i8,
            user_param: *mut c_void,
        )>;

        // Opaque types, pointed to by some function parameters.
        // -----------------------------------------------------------------------------

        /// Opaque type. Used as a pointee.
        pub enum GLSync {}

        /// Compatible with OpenCL `cl_context`.
        pub enum CLContext {}

        /// Compatible with OpenCL `cl_event`.
        pub enum CLEvent {}

        // Vendor extension types
        // -----------------------------------------------------------------------------

        #[allow(non_camel_case_types)]
        pub type GLDebugProc_AMD = Option<extern "system" fn(
            id: u32,
            category: GLEnum,
            severity: GLEnum,
            length: i32,
            message: *const i8,
            user_param: *mut c_void,
        )>;
    "#}.as_bytes())?;
    writeln!(dest)
}


pub fn write_wrapper_types(dest: &mut impl Write, enums: &SortedEnums) -> io::Result<()> {
    let enums = &enums.standard;

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

        impl core::ops::Not for GLBitfield { type Output = Self; fn not(self) -> Self::Output { GLBitfield(!self.0) } }
        impl core::ops::Not for &'_ GLBitfield { type Output = GLBitfield; fn not(self) -> Self::Output { GLBitfield(!self.0) } }
        impl core::ops::Not for &'_ mut GLBitfield { type Output = GLBitfield; fn not(self) -> Self::Output { GLBitfield(!self.0) } }

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
        let key = parse_value(&e.value);
        let val = String::from("GL_") + &e.ident;
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


pub fn write_enum_values(dest: &mut impl Write, enums: &SortedEnums) -> io::Result<()> {
    let enum_sets = &[("GLEnum", &enums.standard), ("GLBitfield", &enums.bitfield)];

    for &(typename, enums) in enum_sets {
        for e in enums {
            let ident = rename_enum_variant(&e.ident);
            let value = &e.value;

            // FIXME: This is (sorta) temporary. Right now, `rename_enum_variant` does nothing; no point going from
            // `UPPER_SNAKE` to `UPPER_SNAKE`. Once enums are removed from being global constants and instead turned
            // into actual enums, that function will start converting cases properly.
            //
            // The issue at hand applies regardless of what `rename_enum_variant` is doing, though. The reason we need
            // this is because there are some names we want to keep in not-quite upper snake case. For example,
            // `FLOAT_MAT2x3`. These are the only ones that need a linter toggle, so we only do it sometimes.
            //
            // So, in the future, when we eventually do have proper enums, we'll still need this. It just won't
            // necessarily be the same check and same linter toggle. That is, unless we manage to find a way to
            // auto-rename all enum variants in such a way that they're perfectly Rust-compatible.
            if ident.chars().any(|c| c.is_lowercase()) {
                write!(dest, "#[allow(non_upper_case_globals)] ")?;
            }

            write!(dest, "pub const {ident}: {typename} = {typename}({value});\n")?;
        }

        writeln!(dest)?;
    }

    // Write special values (except the booleans, we don't need those)
    for e in enums.other.iter().filter(|e| e.ty != "GLboolean") {
        let ident = rename_enum_variant(&e.ident);
        let value = &e.value;
        let ty = rename_xml_type(&e.ty);
        dest.write_fmt(format_args!("pub const {ident}: {ty} = {value};\n"))?;
    }

    Ok(())
}

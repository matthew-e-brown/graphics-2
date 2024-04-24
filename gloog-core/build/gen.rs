use std::borrow::Cow;
use std::collections::BTreeSet;
use std::io::{self, Write};

use gl_generator::{Api, Binding, Cmd, Registry};
use indoc::{indoc, writedoc};

use crate::rename::{rename_function, rename_lib_type, rename_parameter};
use crate::STRUCT_NAME;


/// Output raw a `GLenum` for all applicable types in the registry.
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


/// Output the declaration for the struct of function pointers.
pub fn write_struct_decl(registry: &Registry, dest: &mut impl Write) -> io::Result<()> {
    writedoc!(
        dest,
        r#"
            /// A collection of loaded OpenGL function pointers.
            pub struct {} {{
        "#,
        STRUCT_NAME
    )?;

    for cmd in &registry.cmds {
        writeln!(dest, "    {new_name}: VoidPtr,", new_name = rename_function(&cmd.proto.ident))?;
    }

    writeln!(dest, "}}")?;
    Ok(())
}


/// Output the constructor function for the function pointer struct.
pub fn write_struct_ctor(registry: &Registry, dest: &mut impl Write) -> io::Result<()> {
    writeln!(dest, "impl {STRUCT_NAME} {{")?;

    // Just cuz there's a lot of `{}` in here, it'd be annoying to use `writedoc` on them.
    let init_fn_str = indoc! {r#"
        /// Loads all `OpenGL` function pointers using the provided function to query for individual function pointers
        /// one by one.
        ///
        /// In the event that loading a function pointer fails, this function returns `Err(&str)` containing the name of
        /// the function/symbol that failed to load. A function pointer "fails to load" if `loader_fn` does not return a
        /// non-null pointer after attempting all fallbacks.
        ///
        /// See the documentation of [`InitFailureMode`] for more details.
        pub unsafe fn load(
            mut loader_fn: impl FnMut(&'static str) -> *const c_void,
            failure_mode: InitFailureMode,
        ) -> Result<Self, &'static str> {
            /// Loads an OpenGL function from its symbol name, falling back to the ones in the list (in order) if it
            /// can't be found.
            fn load_ptr(
                mut loader_fn: impl FnMut(&'static str) -> *const c_void,
                name: &'static str,
                fallbacks: &[&'static str],
                failure_mode: InitFailureMode,
            ) -> Result<VoidPtr, &'static str> {
                let mut ptr = loader_fn(name);
                if ptr.is_null() {
                    for &name in fallbacks {
                        ptr = loader_fn(name);
                        if !ptr.is_null() {
                            break;
                        }
                    }
                }

                if !ptr.is_null() {
                    Ok(ptr)
                } else {
                    match failure_mode {
                        InitFailureMode::Abort => Err(name),
                        InitFailureMode::ContinueSilently => Ok(std::ptr::null()),
                        InitFailureMode::WarnAndContinue => {
                            if fallbacks.len() == 0 {
                                log::warn!("failed to load function pointer for {name:?}");
                            } else {
                                log::warn!("failed to load function pointer for {name:?} with fallbacks {fallbacks:?}");
                            }

                            Ok(std::ptr::null())
                        },
                    }
                }
            }

            Ok(Self {
    "#};

    dest.write_all(init_fn_str.as_bytes())?;

    // Now, load them all!!
    for cmd in &registry.cmds {
        let raw_name = &cmd.proto.ident[..];
        let new_name = rename_function(raw_name);

        // This is the part that actually loads the raw function pointer by symbol name---so we need to be careful to
        // get the name right!
        let load_name = match registry.api {
            Api::Gl | Api::GlCore | Api::Gles1 | Api::Gles2 | Api::Glsc2 => format!("\"gl{raw_name}\""),
            Api::Glx => format!("\"glX{raw_name}\""),
            Api::Wgl => format!("\"wgl{raw_name}\""),
            Api::Egl => format!("\"egl{raw_name}\""),
        };

        let fallbacks = registry
            .aliases
            .get(raw_name)
            .and_then(|aliases| {
                // Surround each with `"...",` --> surround the whole thing with `&[...]` --> collect into a string
                let names = aliases.iter().flat_map(|alias| ["\"", &alias[..], "\","].into_iter());
                let bits = Some("&[").into_iter().chain(names).chain(Some("]").into_iter());
                Some(Cow::Owned(bits.collect()))
            })
            .unwrap_or("&[]".into());

        writeln!(dest, "        {new_name}: load_ptr(&mut loader_fn, {load_name}, {fallbacks}, failure_mode)?,")?;
    }

    writeln!(dest, "    }})")?; // Close Ok(Self {...})
    writeln!(dest, "}}\n}}")?; // Close fn and impl
    Ok(())
}

/// Write the `impl` block for the function pointer struct, where the raw void-pointer dereferences/calls are performed.
pub fn write_struct_impl(registry: &Registry, dest: &mut impl Write) -> io::Result<()> {
    // Doesn't need any `write!` formatting
    let macro_str = indoc! {r#"
        /// Casts (transmutes) a void pointer into a function pointer with the given signature so that it may be called.
        macro_rules! cast {
            ($self:ident.$name:ident($($p_ty:ty),*) -> $r_ty:ty) => {
                // SAFETY: the functions calling this macro are all unsafe. Safety responsibilities are upheld by the
                // caller.
                ::core::mem::transmute::<VoidPtr, extern "system" fn($($p_ty),*) -> $r_ty>($self.$name)
            };
        }
    "#};
    dest.write_all(macro_str.as_bytes())?;

    writeln!(dest, "\nimpl {STRUCT_NAME} {{")?;

    for cmd in &registry.cmds {
        let new_name = rename_function(&cmd.proto.ident);
        let ret_type = rename_lib_type(&cmd.proto.ty);
        let fn_cast = make_fn_cast(&cmd);
        let args = make_args(&cmd.params);

        write!(dest, "    pub unsafe fn {new_name}")?;

        if cmd.params.len() == 0 {
            write!(dest, "(&self)")?;
        } else {
            write!(dest, "(&self, {params})", params = make_params(&cmd.params))?;
        }

        if ret_type != "()" {
            write!(dest, " -> {ret_type}")?;
        }

        writeln!(dest, " {{ ({fn_cast})({args}) }}")?;
    }

    writeln!(dest, "}}")?; // Close impl
    Ok(())
}

fn make_params(bindings: &[Binding]) -> String {
    bindings
        .iter()
        .map(|binding| {
            let ident = rename_parameter(&binding.ident);
            let ty = rename_lib_type(&binding.ty);
            format!("{}: {}", ident, ty)
        })
        .collect::<Vec<_>>()
        .join(", ")
}

fn make_args(bindings: &[Binding]) -> String {
    bindings
        .iter()
        .map(|binding| rename_parameter(&binding.ident))
        .collect::<Vec<_>>()
        .join(", ")
}

fn make_fn_cast(cmd: &Cmd) -> String {
    let ident = rename_function(&cmd.proto.ident);
    let ret_ty = rename_lib_type(&cmd.proto.ty);
    let params = cmd
        .params
        .iter()
        .map(|binding| rename_lib_type(&binding.ty))
        .collect::<Vec<_>>()
        .join(", ");
    format!("cast!(self.{ident}({params}) -> {ret_ty})")
}

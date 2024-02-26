use std::borrow::Cow;
use std::io::{self, Write};

use gl_generator::{Api, Binding, Cmd, Registry};
use indoc::{indoc, writedoc};

use crate::rename::{rename_function, rename_lib_type, rename_parameter};
use crate::STRUCT_NAME;


/// Output the declaration for the struct of function pointers.
pub fn write_struct_decl(registry: &Registry, dest: &mut impl Write) -> io::Result<()> {
    writedoc!(dest,
        r#"
            /// How [`{0}::init`] should behave when a function pointer cannot be loaded.
            ///
            /// Be careful with using options other than `Abort`. They should be considered `unsafe`. See their
            /// documentation for more information.
            #[derive(Debug, Clone, Copy)]
            pub enum InitFailureMode {{
                /// Any single function pointer failing to load will cause [`{0}` initialization][{0}] to stop and
                /// return an [`Err`].
                ///
                /// As far as Rust safety guarantees go, **this is the only safe option.** All other options will result
                /// in `{0}` being partially uninitialized, which violates Rust's initialization invariant.
                Abort,
                /// When a function pointer fails to load, a warning will [be logged][https://docs.rs/log] and
                /// initialization will continue. The function pointer will be left as [`null`][std::ptr::null] instead.
                ///
                /// **This option is unsafe.** If a pointer fails to load, [`{0}`] will be left partially uninitialized,
                /// which violates Rust's initialization invariant.
                WarnAndContinue,
                /// When a function pointer fails to load, initialization will continue as if it did not. The function
                /// pointer will be left as [`null`][std::ptr::null] instead.
                ///
                /// **This option is unsafe.** If a pointer fails to load, [`{0}`] will be left partially uninitialized,
                /// which violates Rust's initialization invariant.
                ContinueSilently,
            }}

        "#,
        STRUCT_NAME
    )?;

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

    writedoc!(
        dest,
        r#"
            /// Load all `OpenGL` function pointers using the given function to load function pointers.
            ///
            /// This function returns `Err(&str)` in the event that loading a function fails. The returned string is the
            /// name of the function/symbol that failed to load. A function "fails to load" if the `loader_fn` does not
            /// return a non-null pointer after attempting all fallbacks.
        "#
    )?;

    // Just cuz there's a lot of `{}` in here, it'd be annoying to use `writedoc` on them.
    let init_fn_str = indoc! {r#"
        /// Loads all OpenGL function pointers. See [`InitFailureMode`] for more information.
        pub unsafe fn init(
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
        /// Casts (transmutes) a void pointer into a function pointer with the given arguments and
        macro_rules! cast {
            ($self:ident.$name:ident($($p_ty:ty),*) -> $r_ty:ty) => {
                // SAFETY: the functions doing this transmute are all unsafe; it's up to the caller to
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

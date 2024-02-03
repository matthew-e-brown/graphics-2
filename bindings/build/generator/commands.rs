use std::borrow::Cow;
use std::io::{self, Write};

use gl_generator::{Api, Binding, Cmd, Registry};
use indoc::indoc;

use crate::rename::{rename_function, rename_lib_type, rename_parameter};


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

fn make_fn_ptr(cmd: &Cmd) -> String {
    let mut result = String::from("extern \"system\" fn(");

    let ret_ty = rename_lib_type(&cmd.proto.ty);
    let params = cmd
        .params
        .iter()
        .map(|binding| rename_lib_type(&binding.ty))
        .collect::<Vec<_>>()
        .join(", ");

    result += &params;
    result += ")";

    if ret_ty != "()" {
        result += " -> ";
        result += &ret_ty;
    }

    result
}


pub fn write_gl_struct(registry: &Registry, dest: &mut impl Write) -> io::Result<()> {
    // Create the OpenGL context struct
    #[rustfmt::skip]
    dest.write_all(indoc! {r#"
        #[allow(unused_imports)] use core::mem::transmute;
        #[allow(dead_code)] type VoidPtr = *const core::ffi::c_void;

        /// An abstraction over an OpenGL context.
        ///
        /// This struct _isn't really_ an "OpenGL context;" really, it is a collection of loaded function pointers for use
        /// in the current thread.
        pub struct GLContext {
    "#}.as_bytes())?;

    for cmd in &registry.cmds {
        let ident = rename_function(&cmd.proto.ident);
        writeln!(dest, "    {ident}: VoidPtr,")?;
    }

    writeln!(dest, "}}")?;
    Ok(())
}


pub fn write_gl_struct_ctor(registry: &Registry, dest: &mut impl Write) -> io::Result<()> {
    writeln!(dest, "impl GLContext {{")?;

    #[rustfmt::skip]
    dest.write_all(indoc! {r#"
        /// Load all `OpenGL` function pointers using the given function to load function pointers.
        ///
        /// ```ignore
        /// let gl = GLContext::init(|f| glfw.get_proc_address(f));
        /// ```
        ///
        /// This function returns `Err(&str)` in the event that loading a function fails. The returned string is the
        /// name of the function/symbol that failed to load. A function "fails to load" if the `loader_fn` does not
        /// return a non-null pointer after attempting all fallbacks.
        pub fn init(mut loader_fn: impl FnMut(&'static str) -> *const c_void) -> Result<Self, &'static str> {
            /// The function that actually calls the loader function with the final function name.
            fn load_ptr<F>(mut loader_fn: F, name: &'static str, fallbacks: &[&'static str]) -> Result<VoidPtr, &'static str>
            where
                F: FnMut(&'static str) -> *const c_void,
            {
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
                    Err(name)
                }
            }

            // SAFETY: We transmute all of the loaded function pointers here, although we don't do any validation. This
            // is fine, since the user-facing implementations will also be marked as unsafe; they're not really unsafe
            // until we go to call them. This just changes where the transmutation happens.
            Ok(Self {
    "#}.as_bytes())?;

    // Now, load them all!!
    for cmd in &registry.cmds {
        let raw_name = &cmd.proto.ident[..];
        let new_name = rename_function(raw_name);

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
                // Surround each with quotes and a comma
                let names = aliases.iter().flat_map(|alias| ["\"", &alias[..], "\","].into_iter());
                // Then surround the entire iterator with &[]
                let bits = Some("&[").into_iter().chain(names).chain(Some("]").into_iter());
                // Then collect the entire thing into a single string
                Some(Cow::Owned(bits.collect()))
            })
            .unwrap_or("&[]".into());

        writeln!(dest, "        {new_name}: load_ptr(&mut loader_fn, {load_name}, {fallbacks})?,")?;
    }

    writeln!(dest, "    }})")?; // Close Ok(Self {...})
    writeln!(dest, "}}\n}}")?; // Close fn and impl
    Ok(())
}


pub fn write_gl_struct_impl(registry: &Registry, dest: &mut impl Write) -> io::Result<()> {
    writeln!(dest, "impl GLContext {{")?;

    for cmd in &registry.cmds {
        writeln!(
            dest,
            "    pub unsafe fn {new_name}(&self, {params}){ret_ty} {{ (transmute::<VoidPtr, {fn_ptr}>(self.{new_name}))({args}) }}",
            new_name = rename_function(&cmd.proto.ident[..]),
            params = make_params(&cmd.params),
            ret_ty = {
                let ty = rename_lib_type(&cmd.proto.ty);
                if ty == "()" { "".into() } else { Cow::Owned(format!(" -> {ty}")) }
            },
            fn_ptr = make_fn_ptr(&cmd),
            args = make_args(&cmd.params)
        )?;
    }

    writeln!(dest, "}}")?; // Close impl
    Ok(())
}

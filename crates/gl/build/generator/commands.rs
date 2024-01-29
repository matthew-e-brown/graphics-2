use std::io::{self, Write};

use gl_generator::Registry;
use indoc::indoc;

use crate::rename::{rename_function, rename_lib_type, rename_parameter};


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


pub fn write_gl_struct(registry: &Registry, dest: &mut impl Write) -> io::Result<()> {
    // Create the OpenGL context struct
    #[rustfmt::skip]
    dest.write_all(indoc! {r#"
        /// An abstraction over an OpenGL context.
        ///
        /// This struct _isn't really_ an "OpenGL context;" really, it is a collection of loaded function pointers for use
        /// in the current thread.
        #[allow(dead_code)]         // temp
        pub struct GLContext {
    "#}.as_bytes())?;

    for cmd in &registry.cmds {
        let ident = rename_function(&cmd.proto.ident);
        let ret_ty = rename_lib_type(&cmd.proto.ty).unwrap_or_else(|| panic!("unknown typename: {}", cmd.proto.ty));
        let params = cmd.params.iter().map(|param| {
            let ident = rename_parameter(&param.ident);
            let ty = rename_lib_type(&param.ty).unwrap_or_else(|| panic!("unknown typename: {}", param.ty));
            (ident, ty)
        });

        let fn_ptr = make_fn_ptr(params, &ret_ty);
        writeln!(dest, "    {ident}: Option<{fn_ptr}>,")?;
    }

    writeln!(dest, "}}")?;
    Ok(())
}

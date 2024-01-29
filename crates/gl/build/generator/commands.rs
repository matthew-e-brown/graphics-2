use std::io::{self, Write};

use convert_case::Case;
use gl_generator::Registry;
use indoc::indoc;

use crate::rename::{convert_ident, lib_type_to_rs};


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

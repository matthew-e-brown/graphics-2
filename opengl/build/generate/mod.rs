mod funcs;
mod types;

use std::io::{self, Write};

use gl_generator::Registry;
use indoc::writedoc;

use self::funcs::{write_struct_decl, write_struct_ctor, write_struct_impl};
use self::types::{write_enum_values, write_types_module};


/// Write the OpenGL bindings into
pub fn write_bindings<W: Write>(registry: &Registry, dest: &mut W) -> io::Result<()> {
    write_types_module(dest)?;
    writeln!(dest)?;

    writedoc!(dest, r#"
        #[allow(dead_code)] use self::types::*;
        #[allow(dead_code)] use core::ffi::c_void;
        #[allow(dead_code)] type VoidPtr = *const c_void;
    "#)?;
    writeln!(dest)?;

    write_enum_values(registry, dest)?;
    writeln!(dest)?;

    write_struct_decl(registry, dest)?;
    writeln!(dest)?;

    write_struct_ctor(registry, dest)?;
    writeln!(dest)?;

    write_struct_impl(registry, dest)?;

    Ok(())
}

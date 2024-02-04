mod funcs;
mod types;

use std::io::{self, Write};

use gl_generator::Registry;

use self::funcs::{write_struct_decl, write_struct_ctor, write_struct_impl};
use self::types::{write_enum_values, write_type_aliases, write_wrapper_types, SortedEnums};


/// Write the OpenGL bindings into
pub fn write_bindings<W: Write>(registry: &Registry, dest: &mut W) -> io::Result<()> {
    // Sort through and reformat all of the enums first, since multiple places need those.
    let enums = SortedEnums::from_registry(registry);

    writeln!(dest, "pub mod types {{")?;
    write_type_aliases(dest)?;
    write_wrapper_types(dest, &enums)?;
    writeln!(dest, "}}\n")?;

    writeln!(dest, "#[allow(unused_imports)] use self::types::{{GLEnum, GLBitfield}};")?;
    writeln!(dest, "#[allow(unused_imports)] use core::ffi::c_void;\n")?;

    write_enum_values(dest, &enums)?;
    writeln!(dest)?;

    write_struct_decl(registry, dest)?;
    writeln!(dest)?;

    write_struct_ctor(registry, dest)?;
    writeln!(dest)?;

    write_struct_impl(registry, dest)?;

    Ok(())
}

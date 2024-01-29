mod commands;
mod types;

use std::io::{self, Write};

use gl_generator::{Generator, Registry};

use self::commands::write_gl_struct;
use self::types::{write_enum_values, write_type_aliases, write_wrapper_types, SortedEnums};


pub struct StructGenerator;

impl Generator for StructGenerator {
    fn write<W: Write>(&self, registry: &Registry, dest: &mut W) -> io::Result<()> {
        // Type definitions for custom wrapper types, as well as values for those constants.
        let enums = SortedEnums::from_registry(registry);

        writeln!(dest, "pub mod types {{")?;
        write_type_aliases(dest)?;
        write_wrapper_types(&enums, dest)?;
        writeln!(dest, "}}\n")?;

        writeln!(dest, "#[allow(unused_imports)]")?;
        writeln!(dest, "use self::types::{{GLEnum, GLBitfield}};\n")?;
        write_enum_values(&enums, dest)?;
        writeln!(dest)?;

        write_gl_struct(registry, dest)?;

        Ok(())
    }
}

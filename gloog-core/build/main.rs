mod gen;
mod rename;

use std::env;
use std::fs::File;
use std::io::{self, Write};
use std::path::PathBuf;

use gl_generator::{Api, Fallbacks, Profile, Registry};


/// The name of the outputted struct.
///
/// Note that there are locations within the main crate (doc comments) that refer to this struct's name. If this
/// constant ever changes, make sure to change it there, too.
const STRUCT_NAME: &'static str = "GLPointers";


pub fn main() -> io::Result<()> {
    // Parse the registry for OpenGL Core 4.6 bindings
    let registry = Registry::new(Api::Gl, (4, 6), Profile::Core, Fallbacks::All, []);

    // Get path to Cargo 'target' directory
    let dest_path = env::var("OUT_DIR").expect("OUT_DIR environment variable should be set");
    let dest_path = PathBuf::from_iter([&dest_path, "bindings.rs"]);

    // Create file and output bindings
    let mut dest_file = File::create(&dest_path)?;
    write_bindings(&registry, &mut dest_file)?;

    // Only rerun if the build directory has changes, not if anything in the library changes
    println!("cargo:rerun-if-changed=build");

    Ok(())
}


fn write_bindings<W: Write>(registry: &Registry, dest: &mut W) -> io::Result<()> {
    gen::write_enum_values(registry, dest)?;
    writeln!(dest)?;

    gen::write_struct_decl(registry, dest)?;
    writeln!(dest)?;

    gen::write_struct_ctor(registry, dest)?;
    writeln!(dest)?;

    gen::write_struct_impl(registry, dest)?;
    writeln!(dest)?;

    Ok(())
}

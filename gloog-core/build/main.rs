mod funcs;
mod rename;
mod types;

use std::env;
use std::fs::File;
use std::io::{self, Write};
use std::path::PathBuf;

use gl_generator::{Api, Fallbacks, Profile, Registry};
use indoc::writedoc;

use self::funcs::{write_struct_ctor, write_struct_decl, write_struct_impl};
use self::types::{write_enum_values, write_types_module};


/// What to call the final outputted struct. Something like `GLContext`, `GLFunctionPointers`, etc.
const STRUCT_NAME: &'static str = "GLPointers";


pub fn main() {
    // Parse the registry for OpenGL Core 4.6 bindings
    let registry = Registry::new(Api::Gl, (4, 6), Profile::Core, Fallbacks::All, []);

    // Get path to Cargo 'target' directory
    let dest_path = env::var("OUT_DIR").expect("OUT_DIR environment variable should be set");
    let dest_path = PathBuf::from_iter([&dest_path, "bindings.rs"]);

    // Create file and output bindings
    let mut dest_file = File::create(&dest_path).expect("could not create bindings.rs file");
    write_bindings(&registry, &mut dest_file).expect("failed to write bindings");

    // Only rerun if the build directory has changes, not if anything in the library changes
    println!("cargo:rerun-if-changed=build");
}


fn write_bindings<W: Write>(registry: &Registry, dest: &mut W) -> io::Result<()> {
    write_types_module(dest)?;
    writeln!(dest)?;

    writedoc!(
        dest,
        r#"
            #[allow(dead_code)] use self::types::*;
            #[allow(dead_code)] use core::ffi::c_void;
            #[allow(dead_code)] type VoidPtr = *const c_void;
        "#
    )?;
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

mod generate;
mod rename;

use std::env;
use std::fs::File;
use std::path::PathBuf;

use gl_generator::{Api, Fallbacks, Profile, Registry};


pub fn main() {
    // Parse the registry for OpenGL Core 4.6 bindings
    let registry = Registry::new(Api::Gl, (4, 6), Profile::Core, Fallbacks::All, []);

    // Get path to Cargo 'target' directory
    let dest_path = env::var("OUT_DIR").expect("OUT_DIR environment variable should be set");
    let dest_path = PathBuf::from_iter([&dest_path, "bindings.rs"]);

    // Create file and output bindings
    let mut dest_file = File::create(&dest_path).expect("could not create bindings.rs file");
    generate::write_bindings(&registry, &mut dest_file).expect("failed to write bindings");

    // Only rerun if the build directory has changes, not if anything in the library changes
    println!("cargo:rerun-if-changed=build");
}

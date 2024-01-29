mod generator;
mod rename;

use std::env;
use std::fs::File;
use std::path::Path;

use gl_generator::{Api, Fallbacks, Profile, Registry};


pub fn main() {
    // Generate bindings
    let registry = Registry::new(Api::Gl, (4, 6), Profile::Core, Fallbacks::All, []);

    // eprintln!("{:#?}", registry);

    // Create a file to write to
    let dest = env::var("OUT_DIR").expect("OUT_DIR environment variable should be set");
    let dest = Path::new(&dest).join("bindings.rs");
    let mut file = File::create(dest).expect("failed to create bindings.rs in OUT_DIR");

    registry.write_bindings(generator::StructGenerator, &mut file)
        .expect("failed to write bindings to buffer");
}

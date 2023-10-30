#![allow(unused)]

use std::env;
use std::fs::File;
use std::path::Path;

use gloog_bindgen::{GLProfile, API};


pub fn main() {
    let dest = env::var("OUT_DIR").expect("OUT_DIR environment variable should be set");
    let dest = Path::new(&dest).join("bindings.rs");
    let mut file = File::create(dest).expect("failed to create bindings.rs in OUT_DIR");
    // let bindings = Spec::load(
    //     API::GL {
    //         profile: GLProfile::Core,
    //         version: (4, 6),
    //     },
    //     [],
    // );
}

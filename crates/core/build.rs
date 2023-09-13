use std::env;
use std::fs::File;
use std::path::Path;


pub fn main() {
    let dest = env::var("OUT_DIR").expect("OUT_DIR environment variable should be set");
    let dest = Path::new(&dest).join("bindings.rs");
    let mut file = File::create(dest).expect("should be able to create file in OUT_DIR");
    gloog_bindgen::write_bindings(&mut file).expect("writing bindings should not fail");
}

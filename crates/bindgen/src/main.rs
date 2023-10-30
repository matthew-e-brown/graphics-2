use gloog_bindgen::{GLProfile, API};

pub fn main() {
    // just run the loader once so we can see test prints
    let mut pretend_file = vec![];

    gloog_bindgen::output_bindings(
        &mut pretend_file,
        API::GL {
            version: (4, 6),
            profile: GLProfile::Core,
        },
        [],
    )
    .unwrap();
}

use gloog_bindgen::{GLProfile, Spec, API};

pub fn main() {
    // just run the loader once so we can see test prints
    let _bindings = Spec::load(
        API::GL {
            profile: GLProfile::Core,
            version: (4, 6),
        },
        [],
    );
}

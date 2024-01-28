mod gen;

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

    registry.write_bindings(gen::StructGenerator, &mut file)
        .expect("failed to write bindings to buffer");
}



// --------------------------------------------------------------

    // Look through the final generated output and rename things (now that I've written a big chunk of this it would
    // probably just be easier to implement `gl_generator::Generator` myself lol)
    // #[rustfmt::skip]
    // let re_rename = Regex::new(r"(?x)(?:
    //     \b(?<struct_name>Gl)\b                          | # 'Gl' struct name
    //     ([a-zA-Z0-9_]+):\ FnPtr(?:,$|::new)             | # struct members, the actual function pointers
    //     \#\[inline\]\ pub\ unsafe\ fn\ ([a-zA-Z0-9_]+)  | # impl function names
    //     self\.([a-zA-Z0-9_]+)\.f                          # calls to self.members function pointers
    // )").unwrap();

    // let mut string = String::from_utf8(buffer).expect("gl_generator returned invalid utf8");
    // string = re_rename
    //     .replace_all(&string, |captures: &Captures| {
    //         // Grab the sole capture group from this match
    //         let mut group_iter = captures.iter().skip(1).filter_map(identity);
    //         let group = group_iter.next().expect("should be at least one capture group");
    //         assert!(group_iter.next().is_none(), "should be at most one capture group");

    //         // Grab the ranges around the captured group so we can include the full string
    //         let full = captures.get(0).unwrap();
    //         let chunk1 = &string[full.start()..group.start()];
    //         let chunk2 = &string[group.end() + 1..full.end()];

    //         // Hardcode 'Gl' -> 'GLContext' replacement; everything else is just for the function names and is just for
    //         match group.as_str() {
    //             "Gl" => format!("{chunk1}GLContext{chunk2}"),
    //             name => format!("{chunk1}{}{chunk2}", name.to_case(Case::Snake)),
    //         }
    //     })
    //     .into_owned();

    // --------------------------------------------------------------

// fn func_name_snake_case(name: &str) -> String {
//     lazy_static! {
//         static ref FUNCTION_SUFFIX: Regex = Regex::new(r"[1234]?(?:b|s|i_?|i64_?|f|d|ub|us|ui|ui64|x)?v?$").unwrap();
//     }

//     const KEEP_TOGETHER: &[&'static str] = &[
//         "Arrays",
//         "Attrib",
//         "Box",
//         "Buffers",
//         "Elements",
//         "Enabled",
//         "End",
//         "Feedbacks",
//         "Fixed",
//         "Framebuffers",
//         "Index",
//         "Indexed",
//         "Indexed64",
//         "Indices",
//         "Lists",
//         "Minmax",
//         "Matrix",
//         "Names",
//         "Pipelines",
//         "Pixels",
//         "Queries",
//         "Rects",
//         "Renderbuffers",
//         "Samplers",
//         "Shaders",
//         "Stages",
//         "Status",
//         "Textures",
//         "Varyings",
//         "Vertex",
//     ];

//     // (also 'EXT', but that's not a vendor)
//     const VENDORS: &[&'static str] = &[
//         "ARB", "NV", "NVX", "ATI", "3DLABS", "SUN", "SGI", "SGIX", "SGIS", "INTEL", "3DFX", "IBM", "MESA", "GREMEDY",
//         "OML", "OES", "PGI", "I3D", "INGR", "MTX",
//     ];

//     const FORCE_APART: &[(&'static str, &'static str)] = &[("Getn", "get_n")];

//     // When we get a function name,
// }

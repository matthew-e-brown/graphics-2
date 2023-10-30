/// Generating bindings from a set of features.
pub mod gen;

/// Parsing the OpenGL specification into a set of features.
pub mod xml;


use std::io::{self, Write};

use roxmltree::Document;


pub fn output_bindings<'e, T: Write>(
    mut _output: T,
    api: API,
    extensions: impl IntoIterator<Item = &'e str>,
) -> io::Result<()> {
    let gl_xml = Document::parse(xml::GL_XML).expect("Unable to parse OpenGL XML spec.");
    let features = xml::loading::load_features(&gl_xml, api, extensions);

    for feature in features {
        println!("{feature:?}");
    }

    Ok(())
}

// impl Spec {
//     /// Load and parse a version of the OpenGL spec from XML source.
//     ///
//     /// Note that the API version is **not** checked for correctness. It is only used for comparison, i.e. used to
//     /// filter out parts of the spec that are for versions below the provided one. If you pass `(0, 0)`, you will be a
//     /// returned a valid, *empty* spec; if you pass `(999, 999)`, you'll get back a spec with just about everything.
//     pub fn load<'a, I: IntoIterator<Item = &'a str>>(api: API, extensions: I) -> Self {
//         let extensions = extensions.into_iter().map(|ext| ext.as_bytes());

//         // Run through the XML once to build the list of features we need
//         let features = build_feature_set(api, extensions);

//         // Run through the spec a second time, this time using the parsed set of features
//         todo!()
//     }


//     pub fn write<T: Write>(&self, mut output: T) -> std::io::Result<()> {
//         Ok(())
//     }
// }


/// A `major.minor` version number.
type Version = (u16, u16);


///
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum API {
    GL { version: Version, profile: GLProfile },
    GLES { version: Version, profile: GLESProfile },
    GLSC { version: Version },
}

impl Default for API {
    fn default() -> Self {
        // OpenGL core, version 4.6 (the current most recent)
        Self::GL {
            version: (4, 6),
            profile: GLProfile::Core,
        }
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GLProfile {
    Core,
    Compatibility,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GLESProfile {
    Common,
    Compatibility,
}

impl API {
    pub const fn version(&self) -> Version {
        match *self {
            API::GL { version, .. } => version,
            API::GLES { version, .. } => version,
            API::GLSC { version } => version,
        }
    }

    pub(crate) fn check_name(&self, api_name: &str) -> bool {
        match (*self, api_name) {
            (API::GL { .. }, "gl") => true,
            (API::GLES { version, .. }, "gles1") if version.0 == 1 => true,
            (API::GLES { version, .. }, "gles2") if version.0 >= 1 => true,
            (API::GLSC { .. }, "glsc2") => true,
            (_, "glcore") => true,
            _ => false,
        }
    }

    pub(crate) fn check_version(&self, api_ver: Version) -> bool {
        api_ver <= self.version()
    }

    pub(crate) fn check_profile(&self, profile_name: &str) -> bool {
        match *self {
            API::GL { profile, .. } => match (profile, profile_name) {
                (GLProfile::Core, "core") => true,
                (GLProfile::Compatibility, "compatibility") => true,
                _ => false,
            },
            API::GLES { profile, .. } => match (profile, profile_name) {
                (GLESProfile::Common, "common") => true,
                (GLESProfile::Compatibility, "compatibility") => true,
                _ => false,
            },
            API::GLSC { .. } => panic!("GLSC has no 'profiles'"), // TODO: proper Results?
        }
    }
}

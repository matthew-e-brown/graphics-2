/// Generating bindings from a set of features.
pub mod gen;

/// Parsing the OpenGL specification into a set of features.
pub mod xml;


use std::io::{self, Write};

use roxmltree::Document;
use xml::loading::FeatureSet;
use xml::parsing::Registry;


pub fn generate_bindings<'e, T, E>(mut output: T, api: API, extensions: E) -> io::Result<()>
where
    T: Write,
    E: IntoIterator<Item = &'e str>,
{
    let gl_xml = Document::parse(xml::GL_XML).expect("Unable to parse OpenGL XML spec.");
    let features = FeatureSet::from_xml(&gl_xml, api, extensions);
    let registry = Registry::from_feature_set(&gl_xml, &features);
    Ok(())
}


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

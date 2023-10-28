/// Generating bindings from a parsed and filtered OpenGL specification.
pub mod gen;

/// Parsing the OpenGL specification into a set of features.
pub mod xml;


use std::collections::{HashMap, HashSet};
use std::io::Write;

use self::xml::parsing::{self, ByteStr, Command, Enum};


/// A parsed version of the OpenGL XML specification, including/excluding the appropriate types for a specific version
/// and set of extensions, ready to be written to Rust bindings.
#[derive(Debug, Clone)]
pub struct Spec {
    /// Function bindings to output based on commands like `glVertexAttribPointer`.
    pub commands: HashSet<Command>,
    /// A mapping of GL alias to C type; `GLenum` -> `unsigned int`.
    pub type_defs: HashMap<ByteStr<'static>, ByteStr<'static>>,
    /// Global constants like `GL_TRIANGLE_FAN`. All of them belong to some group, and they will be grouped into actual
    /// Rust enums for our purposes.
    pub regular_enum_groups: HashMap<ByteStr<'static>, Vec<Enum>>,
    /// Global constants like `GL_COLOR_BUFFER_BIT`; almost the same as the regular enums, but these are meant to be
    /// OR'd together to pass multiple flags at once.
    pub bitmask_enum_groups: HashMap<ByteStr<'static>, Vec<Enum>>,
}


impl Spec {
    /// Load and parse a version of the OpenGL spec from XML source.
    ///
    /// Note that the API version is **not** checked for correctness. It is only used for comparison, i.e. used to
    /// filter out parts of the spec that are for versions below the provided one. If you pass `(0, 0)`, you will be a
    /// returned a valid, *empty* spec; if you pass `(999, 999)`, you'll get back a spec with just about everything.
    pub fn load<'a, I: IntoIterator<Item = &'a str>>(api: API, extensions: I) -> Self {
        let extensions = extensions.into_iter().map(|ext| ext.as_bytes());

        // Run through the XML once to build the list of features we need
        let features = parsing::build_feature_set(api, extensions);

        // Run through the spec a second time, this time using the parsed set of features
        todo!()
    }


    pub fn write<T: Write>(&self, mut output: T) -> std::io::Result<()> {
        Ok(())
    }
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
    pub fn version(&self) -> Version {
        match *self {
            API::GL { version, .. } => version,
            API::GLES { version, .. } => version,
            API::GLSC { version } => version,
        }
    }

    pub(crate) fn check_name(&self, api_name: ByteStr) -> bool {
        match (*self, api_name) {
            (API::GL { .. }, b"gl") => true,
            (API::GLES { .. }, b"gles1" | b"gles2") => true,
            (API::GLSC { .. }, b"glsc2") => true,
            (_, b"glcore") => true,
            _ => false,
        }
    }

    /// Checks whether or not the provided API name at the given version should be included in the spec.
    pub(crate) fn check_name_and_version(&self, api_name: ByteStr, api_ver: Version) -> bool {
        match (*self, api_name) {
            // Compatible if our version is the same or newer
            (API::GL { version: self_ver, .. }, b"gl") if api_ver <= self_ver => true,

            // gles1 and gles2 are mutually exclusive; `gles2` supersedes all of `gles1`. So `gles1 1.0` is only
            // compatible if our version starts with a 1 or lower **and** if its version is lower than ours.
            (API::GLES { version: self_ver, .. }, b"gles1") if self_ver.0 == 1 && api_ver <= self_ver => true,

            // gles2 appears for versions 2.x _and_ 3.x and higher.
            (API::GLES { version: self_ver, .. }, b"gles2") if self_ver.0 >= 2 && api_ver <= self_ver => true,

            // Only glsc2 is included in the XML spec; 1.0 only existed as a header file
            // (https://registry.khronos.org/OpenGL/index_sc.php)
            (API::GLSC { version: self_ver, .. }, b"glsc2") if self_ver.0 >= 2 && api_ver <= self_ver => true,

            // Looking at the XML, this branch shouldn't ever happen, but technically 'glcore' should be compatible with
            // everything, I think.
            (_, b"glcore") if api_ver <= self.version() => true,

            // Anything else doesn't match.
            _ => false,
        }
    }

    pub(crate) fn check_profile(&self, profile_name: ByteStr) -> bool {
        match *self {
            API::GL { profile, .. } => match (profile, profile_name) {
                (GLProfile::Core, b"core") => true,
                (GLProfile::Compatibility, b"compatibility") => true,
                _ => false,
            },
            API::GLES { profile, .. } => match (profile, profile_name) {
                (GLESProfile::Common, b"common") => true,
                (GLESProfile::Compatibility, b"compatibility") => true,
                _ => false,
            },
            API::GLSC { .. } => panic!("GLSC has no 'profiles'"), // TODO: proper Results?
            _ => false,
        }
    }
}

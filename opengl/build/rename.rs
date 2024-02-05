//! This module handles renaming the raw types, functions, and values that come out of the OpenGL spec.
//!
//! Mostly, that comes down to converting things from `camelCase` to `snake_case`, `PascalCase`, or `UPPER_SNAKE_CASE`
//! as per Rust conventions.
//!
//! Currently, the functions in this module rename the things returned by [`gl_generator`], not by the actual XML spec.
//! Eventually, a custom XML parser will probably be implemented. (Probably not until after this code is merged into
//! [Gloog](https://github.com/matthew-e-brown/gloog), though).

use std::collections::btree_map::Entry;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Arc, RwLock};

use convert_case::{Boundary, Case, Casing};
use lazy_static::lazy_static;
use regex::Regex;


// Cuz I'm too lazy to type `&'static str` every time lol
type Str = &'static str;


/// Trims the given pattern off of the start of the given slice, shrinking it down in the process.
///
/// Returns the matched pattern for convenience. Helpful when chaining calls to [`Option::or_else`].
fn trim_start_mut<'a, 'p>(str: &mut &'a str, pat: &'p str) -> Option<&'p str> {
    if str.starts_with(pat) {
        *str = &str[pat.len()..];
        Some(pat)
    } else {
        None
    }
}

/// Trims the given pattern off of the end of the given slice, shrinking it down in the process.
///
/// Returns the matched pattern for convenience. Helpful when chaining calls to [`Option::or_else`].
fn trim_end_mut<'a, 'p>(str: &mut &'a str, pat: &'p str) -> Option<&'p str> {
    if str.ends_with(pat) {
        *str = &str[..str.len() - pat.len()];
        Some(pat)
    } else {
        None
    }
}


/// Converts a string slice to a static one.
///
/// # Safety
///
/// This function should be **only** be used for returning slices that refer to data owned by [`lazy_static`] hashmaps.
///
/// We cache the generation of different function, parameter, and type names, etc. so that we don't have to recompute
/// them every time. However, the hashmaps are inside of [`RwLocks`][RwLock], meaning we can only access them through
/// `RwLockReadGuard` and `RwLockWriteGuard` instances. These guard structs have lifetimes tied to the `RwLock`, which
/// makes sense.
///
/// In our case, we will only ever be giving permanent ownership of a [`String`] to these hashmaps, or returning a
/// reference to a `String` we previously inserted. Because each of our hashmaps are private (i.e. local) variables, we
/// know that nobody else will take and drop one from out from under our noses.
unsafe fn str_to_static<'a>(str: &'a str) -> &'static str {
    std::mem::transmute::<&'a str, &'static str>(str)
}


/// Maps a typename from how it appears in the raw OpenGL spec into one of the type aliases supported by this crate.
/// Since this crate now uses its own aliases to map the OpenGL types back to Rust types, this will mostly just
/// re-return the provided string---if it's a supported one.
///
/// Panics if the given typename does not (yet) map to anything supported by this crate.
pub fn rename_xml_type(typename: &str) -> &'static str {
    // cspell:disable
    return match typename {
        // Map known values just back to themselves (but with static lifetime)
        "GLboolean" => "GLboolean",
        "GLbyte" => "GLbyte",
        "GLubyte" => "GLubyte",
        "GLchar" => "GLchar",
        "GLshort" => "GLshort",
        "GLushort" => "GLushort",
        "GLint" => "GLint",
        "GLuint" => "GLuint",
        "GLfixed" => "GLfixed",
        "GLint64" => "GLint64",
        "GLuint64" => "GLuint64",
        "GLsizei" => "GLsizei",
        "GLenum" => "GLenum",
        "GLintptr" => "GLintptr",
        "GLsizeiptr" => "GLsizeiptr",
        "GLsync" => "GLsync",
        "GLbitfield" => "GLbitfield",
        "GLhalf" => "GLhalf",
        "GLfloat" => "GLfloat",
        "GLclampf" => "GLclampf",
        "GLdouble" => "GLdouble",
        "GLclampd" => "GLclampd",
        // We don't use the `GLvoid` type, may as well just use
        "GLvoid" => "core::ffi::c_void",
        "GLDEBUGPROC" => "types::GLDebugProc",
        "GLDEBUGPROCARB" => "types::GLDebugProc",
        "GLDEBUGPROCKHR" => "types::GLDebugProc",
        "GLDEBUGPROCAMD" => "types::GLDebugProc_AMD",
        other if other.ends_with("NV") => rename_xml_type(&other[..other.len() - 2]),
        other if other.ends_with("ARB") => rename_xml_type(&other[..other.len() - 3]),
        other if other.ends_with("EXT") => rename_xml_type(&other[..other.len() - 3]),
        other => unimplemented!("unsupported typename: {other}"),
    };
    // cspell:enable
}


/// Converts a typename from how it appears after being parsed by [`gl_generator`] into one for usage by this crate.
///
/// The ones from `gl_generator` are the ones that look like `__gl_imports::raw::c_ushort`, `types::GLuint`, and so on.
///
/// Panics if the given typename does not (yet) map to anything supported by this crate.
pub fn rename_lib_type(typename: &str) -> &'static str {
    if typename == "()" {
        return "()";
    }

    // -----------------------------------------------------------------------------------------------------------------

    // note to self: see `rename_function` for comments/docs about caching
    lazy_static! {
        static ref CACHE: Arc<RwLock<BTreeMap<String, String>>> = Arc::new(RwLock::new(BTreeMap::new()));
    }

    let cache = CACHE.read().unwrap();
    if let Some(existing) = cache.get(typename).map(|s| s.as_str()) {
        // SAFETY: see function docs.
        return unsafe { str_to_static(existing) };
    }

    std::mem::drop(cache); // drop lock

    // -----------------------------------------------------------------------------------------------------------------

    let mut res = String::new();
    let mut str = typename;

    // Trim off any pointer types and add to our own string
    loop {
        let Some(ptr) = trim_start_mut(&mut str, "*const").or_else(|| trim_start_mut(&mut str, "*mut")) else {
            // Didn't start with either one; done.
            break;
        };

        str = str.trim_start(); // Remove extra space after *const/*mut
        res.push_str(ptr);
        res.push(' ');
    }

    // Map aliases to our types
    if let Some(_) = trim_start_mut(&mut str, "types::") {
        // Type looked like `types::GLtype`; rename the ending to our version.
        res.push_str(rename_xml_type(str));
    } else if let Some(_) = trim_start_mut(&mut str, "__gl_imports::") {
        // Type had `__gl_imports::` at the start. Only known thing with that at the end (in 4.6 core) is a C Void
        // pointer.
        if let Some(_) = trim_start_mut(&mut str, "raw::c_void") {
            res.push_str("c_void");
        } else {
            unimplemented!("unknown type: {typename}");
        }
    } else {
        unimplemented!("unknown type: {typename}");
    }

    // -----------------------------------------------------------------------------------------------------------------

    let mut cache = CACHE.write().unwrap();
    let Entry::Vacant(vacant) = cache.entry(typename.to_owned()) else {
        unreachable!();
    };

    let inserted = vacant.insert(res).as_str();
    // SAFETY: see function docs.
    unsafe { str_to_static(inserted) }
}


/// Converts a function identifier to snake case, taking care to handle OpenGL-specific function endings (e.g., `1fv`).
pub fn rename_function(ident: &str) -> &'static str {
    /// Things that we want to trim off the end of our string before we consider the suffixes. Mostly just the list
    /// of vendors.
    #[rustfmt::skip]
    const KEEP_AFTER_SUFFIX: &[Str] = &[
        // cspell:disable
        "EXT", "ARB", "NV", "NVX", "ATI", "3DLABS", "SUN", "SGI", "SGIX", "SGIS", "INTEL", "3DFX", "IBM", "MESA",
        "GREMEDY", "OML", "OES", "PGI", "I3D", "INGR", "MTX"
        // cspell:enable
    ];

    /// Things that we want to force apart that `convert_case` won't catch. This step happens *after* the case
    /// conversion, so they need to be specified in `lower_snake_case`.
    #[rustfmt::skip]
    const FINAL_REPLACEMENTS: &[(Str, Str)] = &[
        // cspell:disable
        ("getn", "get_n"),
        // cspell:enable
    ];

    #[rustfmt::skip]
    lazy_static! {
        // cspell:disable-next-line
        static ref SUFFIXES: Regex = Regex::new(r"(?:[1234]|[234]x[234]|64)?(?:b|s|i_?|i64_?|f|d|ub|us|ui|ui64|x)?v?$").unwrap();

        /// Valid words at the end of functions.
        ///
        /// These are things that may trip a false positive when searching for function suffixes. For example, the `d`
        /// on the end of `Enabled` matches the suffix regex, but we don't want to split that `d` off of `Enable`.
        static ref NON_SUFFIXES: BTreeSet<Str> = BTreeSet::from_iter([
            // cspell:disable
            "Arrays", "Attrib", "Box", "Buffers", "Elements", "Enabled", "End", "Feedbacks", "Fixed", "Framebuffers",
            "Index", "Indexed", "Indices", "Lists", "Minmax", "Matrix", "Names", "Pipelines", "Pixels", "Queries",
            "Rects", "Renderbuffers", "Samplers", "Shaders", "Stages", "Status", "Textures", "Varyings", "Vertex",
            "1D", "2D", "3D",
            // cspell:enable
        ]);

        /// A cache of all computed function names, to avoid re-parsing and reallocating every time.
        static ref CACHE: Arc<RwLock<BTreeMap<String, String>>> = Arc::new(RwLock::new(BTreeMap::new()));
    }

    // -----------------------------------------------------------------------------------------------------------------

    // Check if this ident has already been renamed before
    let cache = CACHE.read().unwrap();
    if let Some(existing) = cache.get(ident).map(|s| s.as_str()) {
        // SAFETY: see function docs.
        return unsafe { str_to_static(existing) };
    }

    std::mem::drop(cache); // drop lock

    // -----------------------------------------------------------------------------------------------------------------

    let mut vendor = None; // A vendor suffix, if any.
    let mut suffix = None; // One of those OpenGL specifier suffix thingies.
    let mut name = ident; // Name of the function itself. Gets trimmed down as the other two are found.

    // When we get a function name, first check if it ends with any of the given vendor names. If it does, trim the part
    // we care about to that section of the string. Keep track of the function's ending so we can re-add it later.
    for &ending in KEEP_AFTER_SUFFIX.iter() {
        if let Some(_) = trim_end_mut(&mut name, ending) {
            name = name.trim_end_matches("_"); // just in case there's a `_ARB` or something.
            vendor = Some(ending);
            break;
        }
    }

    // Next, check if the function ends with one of our suffixes. A regex made entirely of optional components will
    // always match, so we can safely unwrap.
    let caps = SUFFIXES.captures(name).unwrap();
    let suffix_match = &caps[0];
    if suffix_match.len() > 0 {
        // If we do have a match, look backwards into the string to see if the thing this suffix was attached to is a
        // predetermined non-suffix. This is the reason we do this step before converting case: now, we can reliably
        // iterate backwards until we hit an uppercase letter.
        let upper_idx = name.chars().rev().take_while(|c| !c.is_ascii_uppercase()).count() + 1;
        let last_word = &name[name.len() - upper_idx..];

        // Check to see if the suffix we found was a part of an actual word. If it wasn't, we have a proper suffix that
        // we need to trim off and replace after an underscore. If it was, we leave it be.
        if !NON_SUFFIXES.contains(last_word) {
            suffix = Some(suffix_match);
            name = &name[..name.len() - suffix_match.len()];
        }
    }

    // Zip everything together, converting the name to `lower_snake_case`.
    let mut name = name.from_case(Case::UpperCamel).to_case(Case::Snake);
    if let Some(suffix) = suffix {
        name.push('_');
        name += &suffix.to_lowercase();
    }
    if let Some(vendor) = vendor {
        name.push('_');
        name += &vendor.to_lowercase();
    }

    // Finally, look for any extra replacements manual replacements we want to make and do them. No need to be too fancy
    // here, just replace them manually.
    for &(replace, with) in FINAL_REPLACEMENTS {
        name = name.replace(replace, with);
    }

    // -----------------------------------------------------------------------------------------------------------------

    // Now insert it into the map and return (need to convert ident to a string so we can own it in the static map)
    let mut cache = CACHE.write().unwrap();
    let Entry::Vacant(vacant) = cache.entry(ident.to_owned()) else {
        // We already checked this entry earlier in the function, and returned if it wasn't vacant; it must be vacant.
        unreachable!();
    };

    let inserted = vacant.insert(name).as_str();
    // SAFETY: see function docs.
    unsafe { str_to_static(inserted) }
}


pub fn rename_parameter(ident: &str) -> &'static str {
    lazy_static! {
        static ref CACHE: Arc<RwLock<BTreeMap<String, String>>> = Arc::new(RwLock::new(BTreeMap::from_iter([
            // I prefer 'ty' over 'type_', personally :)
            ("type_", "ty"),
            // If we're trying to avoid collisions with the 'ref' keyword, I think this is a fair thing to name it to:
            // "ref" = "reference value" in both of the only two functions that use it in 4.6 (StencilFunc et al).
            ("ref_", "ref_val"),
            // ------------------
            // cspell:disable
            ("internalformat", "internal_format"),
            ("instancecount", "instance_count"),
            ("baseinstance", "base_instance"),
            ("basevertex", "base_vertex"),
            ("textarget", "tex_target"),
            ("shadertype", "shader_type"),
            ("precisiontype", "precision_type"),
            ("drawcount", "draw_count"),
            ("maxdrawcount", "max_draw_count"),
            ("xoffset", "x_offset"),
            ("yoffset", "y_offset"),
            ("zoffset", "z_offset"),
            ("fixedsamplelocations", "fixed_sample_locations"),
            ("sfail", "s_fail"),
            ("dpfail", "dp_fail"),
            ("dppass", "dp_pass"),
            ("zfail", "z_fail"),
            ("zpass", "z_pass"),
            ("attribindex", "attrib_index"),
            ("relativeoffset", "relative_offset"),
            ("bindingindex", "binding_index"),
            ("renderbuffertarget", "renderbuffer_target"),
            ("bufsize", "buf_size"),
            ("origtexture", "orig_texture"),
            ("sfactor", "s_factor"),
            ("dfactor", "d_factor"),
            ("sfactorRGB", "s_factor_rgb"),
            ("dfactorRGB", "d_factor_rgb"),
            ("sfactorAlpha", "s_factor_alpha"),
            ("dfactorAlpha", "d_factor_alpha"),
            // cspell:enable
        ].into_iter().map(|(k, v)| (k.to_owned(), v.to_owned()))))); // ((((((lol))))))
    }

    let cache = CACHE.read().unwrap();
    if let Some(existing) = cache.get(ident).map(|s| s.as_str()) {
        // SAFETY: see function docs.
        unsafe { str_to_static(existing) }
    } else {
        std::mem::drop(cache); // drop lock

        let new_ident = ident
            .with_boundaries(&[Boundary::LowerUpper, Boundary::Underscore])
            .to_case(Case::Snake);

        let mut cache = CACHE.write().unwrap();
        let Entry::Vacant(vacant) = cache.entry(ident.to_owned()) else {
            unreachable!()
        };

        let inserted = vacant.insert(new_ident).as_str();
        // SAFETY: see function docs
        unsafe { str_to_static(inserted) }
    }
}

//! Parsing the [OpenGL spec][super::GL_XML] into a set of features to be bindgen'ed.

use std::collections::{BTreeSet, HashSet};

use quick_xml::events::{BytesStart, Event};
use quick_xml::Reader;

use crate::{Version, API};


/// A convenience alias for a reference to raw bytes.
///
/// We can't use [`CStr`][std::ffi::CStr] because those are required to be NUL-terminated---we have of bytes from a
/// larger byte-string (`gl.xml`).
pub type ByteStr<'a> = &'a [u8];


/// Parses an `x.y` string into a [tuple of `major, minor` numbers][Version] that can be compared with other version
/// numbers.
fn parse_version(text: ByteStr) -> Version {
    // If there is no '.' in the version string, assume it's a single-number, major-only version (e.g. '2' = 2.0).
    let (maj, min) = match text.iter().position(|c| *c == b'.') {
        Some(idx) => (&text[0..idx], &text[idx + 1..text.len()]),
        None => (&text[..], &b"0"[..]),
    };

    let maj = std::str::from_utf8(maj).expect("OpenGL spec should be valid UTF-8");
    let min = std::str::from_utf8(min).expect("OpenGL spec should be valid UTF-8");

    let maj = u16::from_str_radix(maj, 10).expect("OpenGL spec should only contain valid numbers in version numbers");
    let min = u16::from_str_radix(min, 10).expect("OpenGL spec should only contain valid numbers in version numbers");

    (maj, min)
}

/// Wrapper for panicking, because I'm too lazy to type out the same message every time.
fn invalid_xml(reader: &Reader<ByteStr>, err: quick_xml::Error) -> ! {
    panic!("encountered invalid XML in OpenGL spec at position {}: {:?}", reader.buffer_position(), err)
}

/// Another wrapper, just to shrink the number of lines in match statements (`read_to_end` returns a span, and we want
/// to ignore it, but sticking a semicolon there makes rustfmt put it across multiple lines).
fn read_to_end(reader: &mut Reader<ByteStr>, tag: BytesStart) {
    reader
        .read_to_end(tag.name())
        .expect("all tags in OpenGL spec should close properly");
}


/// The individual items that make up a given subset of the spec.
///
/// These are parsed out of the OpenGL spec by [`build_feature_set`]. For example, OpenGL 1.0 starts with:
///
/// ```xml
/// <require>
///     <type name="GLvoid" comment="No longer used in headers"/>
///     <enum name="GL_DEPTH_BUFFER_BIT"/>
/// </require>
/// ```
///
/// Which are parsed into `Feature::Type(b"GLvoid")` and `Feature::Command(b"GL_DEPTH_BUFFER_BIT")`.
///
/// A set of these is given to [TODO] to know which commands, type definitions, and enums we should bother parsing
/// further.
#[derive(Clone)]
pub enum Feature {
    Command(ByteStr<'static>),
    Type(ByteStr<'static>),
    Enum(ByteStr<'static>),
}

lossy_debug!(enum Feature {
    Command(arg0: "lossy"),
    Type(arg0: "lossy"),
    Enum(arg0: "lossy"),
});


pub fn build_feature_set<'e>(api: API, extensions: impl IntoIterator<Item = ByteStr<'e>>) -> BTreeSet<Feature> {
    // Many `<require>` entries actually just build on top of `<extension>` entries; they have a `Reuse [ext]` comment
    // on them. So, instead of parsing things twice, those `<require>` tags will just get their names pushed in here for
    // us to parse once we get to extensions. We don't care about ordering, so `HashSet` instead of a `BTreeSet`.
    let mut extensions = HashSet::from_iter(extensions.into_iter());
    let mut features = BTreeSet::new();

    // We read through the XML spec multiple times as we build up the list of things we need to generate bindings and
    // types for. Our string is static and constant, so there's not really any harm in restarting our reader multiple
    // times.

    // Start by reading only the `<feature>` tags, which each will add to or remove from the list of features, or will
    // tell us to "reuse" an extension, pushing it into our set.
    let mut reader = Reader::from_str(super::GL_XML);
    loop {
        match reader.read_event() {
            Ok(Event::Start(tag)) => match tag.name().as_ref() {
                b"registry" => continue, // Step into <registry>
                b"feature" => parse_feature(&mut reader, tag, api, &mut extensions, &mut features),
                // completely skip over any other tag; `continue` would unnecessarily step into everything
                _ => read_to_end(&mut reader, tag),
            },
            // Hitting the end of the file means we're finished
            Ok(Event::Eof) => break,
            // We don't care about any other elements yet
            Ok(_) => continue,
            // Shouldn't happen, since the XML comes directly from Khronos and is static, so should always be valid.
            Err(e) => invalid_xml(&reader, e),
        }
    }

    // Next, start again go down to the `<extensions>` tag and pick up the extensions requested either by the user or by
    // a feature.
    let mut reader = Reader::from_str(super::GL_XML);
    loop {
        match reader.read_event() {
            Ok(Event::Start(tag)) => match tag.name().as_ref() {
                b"registry" => continue,   // step into <registry>
                b"extensions" => continue, // step into <extensions>
                b"extension" => parse_extension(&mut reader, tag, api, &extensions, &mut features),
                _ => read_to_end(&mut reader, tag),
            },
            Ok(Event::Eof) => break,
            Ok(_) => continue,
            Err(e) => invalid_xml(&reader, e),
        }
    }

    features
}


fn parse_feature<'e>(
    reader: &mut Reader<ByteStr<'static>>,
    start_tag: BytesStart<'static>,
    api: API,
    extensions: &mut HashSet<ByteStr<'e>>,
    features: &mut BTreeSet<Feature>,
) {
    // Pull the API name and version off of start tag
    let mut feat_api = None;
    let mut feat_ver = None;

    for attr in start_tag.attributes() {
        let attr = attr.expect("OpenGL XML attributes should always be valid");
        match attr.key.as_ref() {
            b"api" => feat_api = Some(attr.value),
            b"number" => feat_ver = Some(attr.value),
            _ => {},
        }
    }

    let feat_api = &feat_api.expect("<feature> elements should always have an 'api' attribute")[..];
    let feat_ver = feat_ver.expect("<feature> elements should always have a 'number' attribute");
    let feat_ver = parse_version(&feat_ver);

    // If this version of this API belongs in the requested feature-set, start parsing it
    if api.check_version(feat_api, feat_ver) {
        // Start looking for <require> tags
        loop {
            match reader.read_event() {
                Ok(Event::Start(tag)) => match tag.name().as_ref() {
                    b"require" => parse_require_remove(reader, tag, api, extensions, features, true),
                    b"remove" => parse_require_remove(reader, tag, api, extensions, features, false),
                    // there should never be any other tags; ignore just in case.
                    _ => read_to_end(reader, tag),
                },
                // If we find a </feature>, we are finished.
                Ok(Event::End(tag)) => match tag.name().as_ref() {
                    b"feature" => break,
                    _ => continue, // ignore other end tags
                },
                Ok(Event::Eof) => panic!("found EOF before closing </feature> tag"),
                Ok(_) => continue,
                Err(e) => invalid_xml(reader, e),
            }
        }
    } else {
        reader.read_to_end(start_tag.name());
    }
}


fn parse_require_remove<'e>(
    reader: &mut Reader<ByteStr<'static>>,
    start_tag: BytesStart<'static>,
    api: API,
    extensions: &mut HashSet<ByteStr<'e>>,
    features: &mut BTreeSet<Feature>,
    is_require: bool,
) {
    if is_require {
        println!("Parsing <require>");
    } else {
        println!("Parsing <remove>");
    }

    for attr in start_tag.attributes() {
        let attr = attr.expect("OpenGL XML attributes should always be valid");
        match attr.key.as_ref() {
            b"api" if !api.check_api(&attr.value) => return,
            b"profile" if !api.check_profile(&attr.value) => return,
            b"comment" if attr.value.starts_with(b"Reuse") => {
                // read until next space or end
                let s = b"Reuse ".len();
                let e = attr.value.iter().skip(s).take_while(|c| !c.is_ascii_whitespace()).count();
                // ------------------
                // This breaks.
                extensions.insert(&attr.value[s..s + e]);
                // - See here: https://github.com/tafia/quick-xml/issues/332
                // - It seems like the lack of lifetime on the `Attribute`` return value gives it the lifetime of the
                //   **BytesStart**'s &self reference, **not** the lifetime of the underlying reader.
                // ------------------
                return;
            },
            _ => {},
        }
    }
}


fn parse_extension<'e>(
    reader: &mut Reader<ByteStr<'static>>,
    start_tag: BytesStart,
    api: API,
    extensions: &HashSet<ByteStr<'e>>,
    features: &mut BTreeSet<Feature>,
) {
    todo!();
}


/// An OpenGL _command_, like `glVertexAttribPointer`.
#[derive(Clone)]
pub struct Command {
    name: ByteStr<'static>,
    params: Vec<CommandParam>,
    return_type: ByteStr<'static>,
    _glx: Option<GLXInfo>,
}

lossy_debug!(struct Command {
    name: "lossy",
    params: "passthrough",
    return_type: "lossy",
    _glx: "passthrough",
});

/// A parameter for a [command][Command], like `GLsizei stride` in `glVertexAttribPointer`.
#[derive(Clone)]
pub struct CommandParam {
    ty: ByteStr<'static>,
    name: ByteStr<'static>,
    kind: Option<ByteStr<'static>>,
}

lossy_debug!(struct CommandParam {
    ty: "lossy",
    name: "lossy",
    kind: "lossy-option",
});

/// Any GLX-related information that appears on a [command][Command].
///
/// These currently isn't used for anything, but it can't hurt to pull it out of the XML for anybody who may want it in
/// the future. :shrug:
#[derive(Clone)]
pub struct GLXInfo {
    opcode: u16,
    ty: ByteStr<'static>,
    name: Option<ByteStr<'static>>,
    comment: Option<ByteStr<'static>>,
}

lossy_debug!(struct GLXInfo {
    opcode: "passthrough",
    ty: "lossy",
    name: "lossy-option",
    comment: "lossy-option",
});


#[derive(Clone)]
pub struct Typedef {}


/// An OpenGL enum.
///
/// OpenGL uses the term "enum" differently than Rust does. In OpenGL, an `enum` is simply a value with a name
/// associated with it: in Rust, these would each be _variants_ of enums. Since C has no namespaces, they all start with
/// `GL_` to avoid collisions; but C can't group each set of enums together very nicely (e.g., making all the
/// `GL_???_BUFFER_BIT` enums grouped together). Thankfully, `gl.xml` provides the extra metadata of a `group` for each
/// one specifically for the purposes of making higher-level bindings.
///
/// This struct makes no distinction between regular enums and bitmasks: discrete values vs. constituent parts to be
/// OR'd together. The same data from the `<enum>` tag is needed for both, they're just output differently. Different
/// types are stored in a different map during parsing.
#[derive(Clone)]
pub struct Enum {
    name: ByteStr<'static>,
    group: ByteStr<'static>,
    value: ByteStr<'static>,
}

lossy_debug!(struct Enum {
    name: "lossy",
    group: "lossy",
    value: "lossy",
});


/// Implements [`Debug`][std::fmt::Debug] for the given struct or enum by falling back to [`String::from_utf8_lossy`]
/// whenever
macro_rules! lossy_debug {
    // `struct Name { field: dbg_mode, field: dbg_mode }`
    (struct $name:ident { $($field:ident: $dbg_mode:tt),+ $(,)? }) => {
        impl std::fmt::Debug for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                let mut builder = f.debug_struct(stringify!($name));
                $({
                    let field = &self.$field; // the `.` in `self.$field` breaks `$tt`, need to pass a fresh ident
                    builder.field(stringify!($field), &$crate::parsing::lossy_debug!(@ $dbg_mode: field));
                })+
                builder.finish()
            }
        }
    };
    // `enum Name { Variant, Variant, Variant(a0: dbg_mode), Variant(a0: dbg_mode, a1: dbg_mode) }`
    (enum $name:ident { $(   $variant:ident $(( $($field:ident: $dbg_mode:tt),* ))?   ),+ $(,)? }) => {
        impl std::fmt::Debug for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                match self {$(
                    Self::$variant$(( $($field),* ))? => {
                        let mut builder = f.debug_tuple(stringify!($variant));
                        $($(builder.field( $crate::parsing::lossy_debug!(@ $dbg_mode: $field));)*)?
                        builder.finish()
                    }
                ),+}
            }
        }
    };
    (@ "lossy" : $field:ident) => (&String::from_utf8_lossy($field));
    (@ "lossy-option" : $field:ident) => (&$field.map(|s| String::from_utf8_lossy(s)));
    (@ "passthrough"  : $field:ident) => ($field);
}

pub(crate) use lossy_debug;

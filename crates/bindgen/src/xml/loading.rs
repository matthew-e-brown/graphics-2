use std::collections::HashSet;

pub use roxmltree::Document as XmlDocument;
use roxmltree::Node;

use crate::{Version, API};


/// Parses an `x.y` string into a tuple of `major, minor` [version numbers][Version] that can be compared with other
/// version numbers.
fn parse_version(text: &str) -> Version {
    // If there is no '.' in the version string, assume it's a single-number, major-only version (e.g. '2' = 2.0).
    let (maj, min) = match text.chars().position(|c| c == '.') {
        Some(idx) => (&text[0..idx], &text[idx + 1..text.len()]),
        None => (&text[..], "0"),
    };

    let maj = u16::from_str_radix(maj, 10).expect("OpenGL spec should only contain valid numbers in version numbers");
    let min = u16::from_str_radix(min, 10).expect("OpenGL spec should only contain valid numbers in version numbers");

    (maj, min)
}


#[derive(Debug, Clone, Default)]
pub struct FeatureSet<'a> {
    pub types: HashSet<&'a str>,
    pub enums: HashSet<&'a str>,
    pub commands: HashSet<&'a str>,
}


pub fn load_features<'a, 'input, 'e, E: IntoIterator<Item = &'e str>>(
    gl_xml: &'a XmlDocument<'input>,
    api_config: API,
    extensions: E,
) -> FeatureSet<'a> {
    let extensions = HashSet::<_>::from_iter(extensions.into_iter());

    // There should always be a 'registry' tag as the first child
    let registry = gl_xml.root().first_element_child().unwrap();
    assert_eq!(registry.tag_name().name(), "registry");

    let filter_feature = |node: &Node| {
        // <feature> tags should always have 'api' and 'number' attributes
        let api = node.attribute("api").unwrap();
        let ver = parse_version(node.attribute("number").unwrap());
        api_config.check_name(api) && api_config.check_version(ver)
    };

    let filter_extension = |node: &Node| {
        let name_attr = node.attribute("name").unwrap();
        let support_attr = node.attribute("supported").unwrap();
        let is_supported = support_attr.split('|').find(|api| api_config.check_name(api)).is_some();
        match (extensions.contains(name_attr), is_supported) {
            (false, _) => false,
            (true, true) => true,
            (true, false) => panic!("Requested extension is unsupported: {name_attr}"),
        }
    };

    // First thing's first, we need to find all the `<feature>` and `<extension>` tags
    let mut features = FeatureSet::default();
    for el in registry.children().filter(|node| node.is_element()) {
        match el.tag_name().name() {
            // Feature tags are one each
            "feature" if filter_feature(&el) => read_feature(api_config, el, &mut features),
            // But the extensions are all within an `extensions` tag
            "extensions" => el
                .children()
                .filter(|node| node.is_element() && filter_extension(node))
                .for_each(|ext| read_feature(api_config, ext, &mut features)),
            _ => continue,
        }
    }

    features
}


fn read_feature<'a, 'input>(api_config: API, feat_tag: Node<'a, 'input>, features: &mut FeatureSet<'a>) {
    for el in feat_tag.children().filter(|node| node.is_element()) {
        match el.tag_name().name() {
            "require" => read_require(api_config, el, features, false),
            "remove" => read_require(api_config, el, features, true),
            other => panic!("unknown element in <feature> or <extension>: {other:?}"),
        }
    }
}


fn read_require<'a, 'input>(api_config: API, req_tag: Node<'a, 'input>, features: &mut FeatureSet<'a>, negate: bool) {
    for el in req_tag.children().filter(|node| node.is_element()) {
        // First check and see if there's an API or profile attribute on this tag; if so, check support for it.
        // Otherwise, it's supported by default.
        if !el.attribute("api").map(|n| api_config.check_name(n)).unwrap_or(true) {
            continue;
        }

        // Do this check after the `name_support`, since in the event that `api="glsc"`, there is no such thing as a
        // profile and we don't wanna run into our own panic.
        if !el.attribute("profile").map(|p| api_config.check_profile(p)).unwrap_or(true) {
            continue;
        }

        let name = el.attribute("name").unwrap();
        let el_type = match el.tag_name().name() {
            "type" => &mut features.types,
            "enum" => &mut features.enums,
            "command" => &mut features.commands,
            other => panic!("unknown element in <require> or <remove>: {other:?}"),
        };

        if negate {
            el_type.remove(&name);
        } else {
            el_type.insert(&name);
        }
    }
}

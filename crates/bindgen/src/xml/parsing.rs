use std::borrow::Cow;
use std::collections::HashMap;

use convert_case::{Case, Casing};
pub use roxmltree::Document as XmlDocument;


use super::loading::FeatureSet;

// cspell:words newtype newtypes ptype

/// Checks if some text is already in the given case, and converts it if isn't.
fn to_case(text: &str, case: Case) -> Cow<'_, str> {
    if text.is_case(case) {
        Cow::Borrowed(text)
    } else {
        Cow::Owned(text.to_case(case))
    }
}


/// A set of Rust types, functions, aliases, and so on that are ready to be output to a file.
#[derive(Debug, Clone, Default)]
pub struct ParsedRegistry<'input> {
    /// Rust function bindings.
    pub functions: Vec<FunctionBinding<'input>>,
    /// Rust 'newtypes'; structs that directly wrap a primitive value. Derived from the `class` attribute on function
    /// prototypes and parameters in the OpenGL spec.
    pub classes: HashMap<Cow<'input, str>, &'static str>,
    /// Bitmask structs, derived from sets of `<enums type="bitmask">` from the OpenGL spec.
    pub bitmasks: HashMap<Cow<'input, str>, Vec<EnumVariant<'input>>>,
    /// Rust enums, derived from enum "groups" from the OpenGL spec.
    pub enums: HashMap<Cow<'input, str>, Vec<EnumVariant<'input>>>,
}


impl<'input> ParsedRegistry<'input> {
    pub fn from_feature_set(gl_xml: &XmlDocument<'input>, feature_set: &FeatureSet) -> Self {
        // There should always be a 'registry' tag as the first child
        let registry_node = gl_xml.root().first_element_child().unwrap();
        assert_eq!(registry_node.tag_name().name(), "registry");

        // Caches of types that we have already parsed, indexed by their raw name in the spec.
        // let mut enum_names = HashMap::<&'input str, Rc<Cow<'input, str>>>::new(); // indexed by `group` attribute
        // let mut class_names = HashMap::<&'input str, Rc<Cow<'input, str>>>::new(); // indexed by `class` attribute

        let mut registry = Self::default();

        // TODO: parse commands before enums. Use the set of 'group' that appear in <proto> and <param> tags to filter
        // the groups that we actually end up needing to generate. I.E. `GL_SHORT` is part of a ton of different groups,
        // but we only need some of them. Could also use the list of found groups to filter the enum groups after the
        // fact, but that seems like it'd waste a lot of unnecessary allocation if we're only going to drop the freshly
        // case-fixed names later.

        for el in registry_node.children().filter(|node| node.is_element()) {
            // The only things we care about are enums and commands; everything else is found within one of these two
            // tags. The only exception being plain types, but we will manually map those to a Rust type anyways.
            match el.tag_name().name() {
                "enums" => {
                    let is_bitmask = el.attribute("type").is_some_and(|attr| attr == "bitmask");

                    // Bitmask enums are going to have their variants as associated constants, and so need to get
                    // UPPER_SNAKE_CASE. Regular enums will become regular enum variants, and so we want them in
                    // PascalCase.
                    let variant_case = is_bitmask.then_some(Case::UpperSnake).unwrap_or(Case::Pascal);

                    // Obviously, if we are in a group of bitmasks then we want to add
                    let enum_map = is_bitmask.then_some(&mut registry.bitmasks).unwrap_or(&mut registry.enums);

                    for enum_node in el.children().filter(|node| node.is_element()) {
                        match enum_node.tag_name().name() {
                            "enum" => (),
                            "unused" => continue,
                            other => panic!("expected <enum> or <unused>, found <{other}>"),
                        }

                        // Every enum has a name. Check if that name occurs in our desired set of enums.
                        let name = enum_node.attribute("name").unwrap();
                        if !feature_set.enums.contains(name) {
                            continue;
                        }

                        // Once we've checked, we can remove the `GL_` prefix.
                        let name = name.trim_start_matches("GL_");

                        // They _should_ also have a value and group. Not all of them have a group, but our current goal
                        // is only to support those that do.
                        let value = enum_node.attribute("value").expect("found <enum> without 'name'");
                        let Some(groups) = enum_node.attribute("group") else {
                            // There's a surprisingly good number of these... we'll have to work out what to do here.
                            // - Some of them are obvious; `GL_CONTEXT_LOST` is supposed to be `ErrorCode`, but it's
                            //   missing one. I could submit PRs for them one-by-one until everything has a proper set
                            //   of groups?
                            // - We could define a set of manual overrides in this file directly.
                            // - Could document them in a Rust doc comment as "currently unsupported"
                            // - Could stick them in an "other" enum.
                            println!("found <enum> without group: {}", enum_node.attribute("name").unwrap());
                            continue;
                        };

                        let variant = EnumVariant {
                            name: to_case(name, variant_case),
                            value,
                        };

                        for group in groups.split(',') {
                            // Most (all?) group names are already in PascalCase, so this _shouldn't_ ever have to
                            // allocate.
                            let group = to_case(group, Case::Pascal);
                            let variants = enum_map.entry(group).or_default();
                            variants.push(variant.clone());
                        }
                    }
                },
                "commands" => {
                    for command_node in el.children().filter(|node| node.is_element()) {
                        // parse proto
                    }
                },
                _ => continue,
            }
        }

        println!("{registry:#?}");

        todo!();
    }
}


#[derive(Debug, Clone)]
pub struct FunctionBinding<'input> {
    pub return_type: Type<'input>,
    pub name: Cow<'input, str>, // converted to Rust casing
    pub gl_name: &'input str,
    pub params: Vec<Parameter<'input>>,
    pub glx_info: GLXInfo<'input>,
    pub aliases: Vec<&'input str>,
}

#[derive(Debug, Clone)]
pub struct Parameter<'input> {
    pub ty: Type<'input>,
    pub name: Cow<'input, str>, // converted to Rust casing
    pub kind: Option<&'input str>,
    pub len: Option<&'input str>,
}

#[derive(Debug, Clone)]
pub struct GLXInfo<'input> {
    pub ty: &'input str,
    pub opcode: &'input str,
    pub name: Option<&'input str>,
    pub comment: Option<&'input str>,
}

#[derive(Debug, Clone)]
pub struct Type<'input> {
    /// What this type actually is.
    pub val: TypeOfType<'input>,
    /// Whether or not this type is a pointer to the given type, and which kind.
    pub ptr: TypeOfPointer,
}

#[derive(Debug, Clone)]
pub enum TypeOfType<'input> {
    /// C `void`, or Rust `()`.
    Void,
    /// A Rust primitive type, from a manually-defined C-to-Rust mapping.
    Primitive(&'static str),
    /// The **name** of one of our to-be-generated types.
    Generated(Cow<'input, str>),
}

#[derive(Debug, Clone)]
pub enum TypeOfPointer {
    NotAPointer,
    ConstPointer,
    MutPointer,
}

#[derive(Debug, Clone)]
pub struct EnumVariant<'input> {
    pub name: Cow<'input, str>,
    pub value: &'input str,
}

// A return type could be be:
// - `void` -> map to `()`
// - A `GLenum`
//   - With a `group` -> map to an enum or bitmask we will output
//   - without a `group` (in rare cases) -> map to whatever `GLenum`'s underlying type is
// - Some other `GL___` type
//   - without a `class` on it -> output that type directly; i.e. several return numbers
//   - with a `class` on it -> map to one of the new-type wrappers we will output
// - It could also be any of these with a pointer type, too

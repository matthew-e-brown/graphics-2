use std::collections::HashMap;

pub use roxmltree::Document as XmlDocument;

use super::loading::FeatureSet;


pub fn parse_features<'a, 'input>(gl_xml: &'a XmlDocument<'input>, features: &FeatureSet<'a>) -> Registry<'a> {
    // Using the list of features we have, run through all the actual definitions and check if they exist in our list of
    // features. If so, actually bother to read that tag and add it to *another* list of things to generate final
    // bindings for.

    todo!()
}


pub struct Registry<'a> {
    pub types: Vec<Typedef<'a>>,
    pub classes: HashMap<&'a str, &'a str>, // name -> underlying type
    pub commands: Vec<Command<'a>>,
    pub bitmasks: HashMap<&'a str, Enum<'a>>, // group name -> enum
    pub enums: HashMap<&'a str, Enum<'a>>, // group name -> enum
}

pub struct Typedef<'a> {
    pub name: &'a str,
    pub value: &'a str,
}

pub struct Command<'a> {
    pub name: &'a str,
    pub params: Vec<CommandParam<'a>>,
    pub return_type: &'a str,
    pub glx_info: GLXInfo<'a>,
    pub aliases: Vec<&'a str>,
}

pub struct CommandParam<'a> {
    pub ty: &'a str,
    pub name: &'a str,
    pub kind: Option<&'a str>,
}

pub struct GLXInfo<'a> {
    pub ty: &'a str,
    pub opcode: &'a str,
    pub name: Option<&'a str>,
    pub comment: Option<&'a str>,
}

pub struct Enum<'a> {
    pub name: &'a str,
    pub value: &'a str,
}

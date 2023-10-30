/// Loading a filtered subset of features from the XML specification.
pub mod loading;

/// Parsing the [loaded set of features][loading] into data structures that represent the final bindings we'll output.
pub mod parsing;

/// The contents of [`gl.xml`](https://github.com/KhronosGroup/OpenGL-Registry/blob/main/xml/gl.xml).
pub const GL_XML: &'static str = include_str!("../../registry/opengl/xml/gl.xml");

pub mod parsing;

/// The contents of [`gl.xml`](https://github.com/KhronosGroup/OpenGL-Registry/blob/main/xml/gl.xml).
pub const GL_XML: &'static str = include_str!("../../registry/opengl/xml/gl.xml");

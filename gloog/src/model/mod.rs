use gloog_core::raw::types::GLuint;
use gloog_core::types::{VertexArrayID, VertexAttribSize, VertexAttribType};
use gloog_core::GLContext;

pub mod mesh;


/// A trait for any struct which represents arbitrary vertex data. Contains functions for describing vertex attributes
/// to OpenGL.
///
/// This struct assumes that vertex data is laid out in an array-of-structs (AOS) format. Struct-of-arrays layouts are
/// not yet supported.
///
/// When configuring vertex attributes, [`std::mem::size_of`] is used to determine vertex stride. The offset of each
/// attribute within each vertex is provided by [`ATTRIBUTE_OFFSETS`][VertexData::ATTRIBUTE_OFFSETS]; a
/// compile-time-constant list of attribute positions and offsets.
///
/// # Safety
///
/// This trait is unsafe because any implementors **must** be `#[repr(C)]`.
pub unsafe trait VertexData: Sized {
    /// The offsets of each of the fields of this struct. Given as a tuple of `(attribindex, relativeoffset)`, as seen
    /// in parameters to
    const ATTRIBUTE_INFO: &'static [VertexAttribInfo];

    // [TODO] Eventually, it would be nice to have some functionality for dynamically generating shaders out of
    // type-system-available data. This trait should then have a function for generating a list of `layout(location =
    // ...)` vertex attributes and whatnot.

    /// Configure a given VAO to match this specific set of vertex attributes. Does not configure any buffer binding
    /// points.
    fn configure_vao_format(&self, gl: &GLContext, vao: VertexArrayID) {
        // [TODO] Switch to using all DSA functions; no more binding!
        gl.bind_vertex_array(vao);

        // Stride is handled when we configure vertex binding points; just the formats for today.
        for attr in Self::ATTRIBUTE_INFO {
            // Determine which `glVertexFormat` function we're going to call, either by using the one given by the user
            // or by guessing based on the attribute type.
            let attrib_coercion = attr.coerce_to.unwrap_or_else(|| match attr.attrib_type {
                VertexAttribType::Float
                | VertexAttribType::HalfFloat
                | VertexAttribType::Fixed
                | VertexAttribType::PackedSignedInts210Rev
                | VertexAttribType::PackedUnsignedInts210Rev
                | VertexAttribType::PackedFloats1011Rev => VertexAttribCoercion::Float,
                VertexAttribType::Byte
                | VertexAttribType::UnsignedByte
                | VertexAttribType::Short
                | VertexAttribType::UnsignedShort
                | VertexAttribType::Int
                | VertexAttribType::UnsignedInt => VertexAttribCoercion::Int,
                VertexAttribType::Double => VertexAttribCoercion::Double,
            });

            // Enable the correct attribute
            gl.enable_vertex_attrib_array(attr.attrib_index);
            match attrib_coercion {
                VertexAttribCoercion::Float => gl.vertex_attrib_format(
                    attr.attrib_index,
                    attr.attrib_size,
                    attr.attrib_type,
                    attr.normalized,
                    // [TODO] Switch all gloog-core functions to using OpenGL type aliases for consistency with wiki and
                    // documentation (except for `bool` <-> `GLboolean`, that one sucks). Probably better to have the
                    // data-type conversion happen all at once at the start/when loading a mesh instead of whenever a
                    // VAOs is configured.
                    attr.rel_offset as usize,
                ),
                VertexAttribCoercion::Int => todo!("[TODO] Make wrapper function for glVertexAttribIFormat"),
                VertexAttribCoercion::Double => todo!("[TODO] Make wrapper function for glVertexAttribLFormat")
            }
        }
    }
}


/// Determines how a given vertex attribute should be passed to OpenGL.
///
/// There exist three functions in OpenGL to define vertex attributes:
///
/// - `glVertexAttribPointer` / `glVertexAttribFormat`,
/// - `glVertexAttribIPointer` / `glVertexAttribIFormat`, and
/// - `glVertexAttribLPointer` / `glVertexAttribLFormat`.
///
/// The first of these will always coerce its arguments into floats. In order to pass an integer,
/// `glVertexAttribIPointer` must be used. This enum allows one to specify which one should be used for a given
/// attribute.
///
/// If `None` is used instead, the attribute is assumed to use the "most sensible" corresponding function, instead of
/// always casting to floats. That is:
///
/// - `Float`, `HalfFloat`, `Fixed`, and the packed ints & packed floats will default to using `AttribFormat`;
/// - `Byte`, `Short`, `Int`, and their unsigned variants will default to using `AttribIFormat`; and
/// - `Double` will default to using `AttribLFormat`.
#[derive(Debug, Clone, Copy)]
pub enum VertexAttribCoercion {
    Float,
    Int,
    Double,
}


#[derive(Debug, Clone, Copy)]
pub struct VertexAttribInfo {
    pub attrib_index: GLuint,
    pub attrib_size: VertexAttribSize,
    pub attrib_type: VertexAttribType,
    pub normalized: bool,
    pub rel_offset: GLuint,
    /// See [`VertexAttribCoercion`].
    pub coerce_to: Option<VertexAttribCoercion>,
}

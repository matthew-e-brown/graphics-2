use gloog_core::raw::types::GLuint;
use gloog_core::types::DrawMode;

use super::VertexData;


// Notes:
// - is `Vec` the best choice? Should I swap for `Arc` later?
// - *Maybe* the idea of a multi-mesh would be good?
//   - Ability to store different meshes together in the same object, though rendered with different draw calls.
//   - Just to make grouping them easier when we get to scene graphs.


/// A simple basic mesh of vertices, agnostic of any material information.
pub enum Mesh<V: VertexData> {
    /// A simple list of triangles. Drawn with `GL_TRIANGLES`.
    Triangle(BasicMesh<V>),

    /// A triangle-fan mesh, wherein a list of vertices define many triangles all fanned around a common starting
    /// vertex. Drawn with `GL_TRIANGLE_FAN`.
    Fan(BasicMesh<V>),

    /// A single triangle-strip mesh, wherein every pair of vertices specifies a new triangle which is adjoined with the
    /// previous triangle's last vertex. Drawn with `GL_TRIANGLE_STRIP`.
    Strip(BasicMesh<V>),

    /// An indexed version of a [triangle mesh][Mesh::Triangle]. Drawn with `GL_TRIANGLES` and an
    /// `ELEMENT_ARRAY_BUFFER`.
    IndexedTriangle(IndexedMesh<V>),

    /// An indexed version of a [triangle-fan mesh][Mesh::Fan]. Drawn with `GL_TRIANGLE_FAN` and an
    /// `ELEMENT_ARRAY_BUFFER`.
    IndexedFan(IndexedMesh<V>),

    /// An indexed version of a [triangle-strip mesh][Mesh::Strip]. Drawn with `GL_TRIANGLE_STRIP` and an
    /// `ELEMENT_ARRAY_BUFFER`.
    IndexedStrip(IndexedMesh<V>),

    /// An indexed version of a [triangle-fan mesh][Mesh::Triangle] that, in addition to an index buffer, uses a
    /// _primitive restart index_ to enable storing multiple triangle fans in a single buffer.
    RestartIndexedFan(RestartIndexedMesh<V>),

    /// An indexed version of a [triangle-strip mesh][Mesh::Strip] that, in addition to an index buffer, uses a
    /// _primitive restart index_ to enable storing multiple triangle strips in a single buffer.
    RestartIndexedStrip(RestartIndexedMesh<V>),
}

impl<V: VertexData> Mesh<V> {
    /// Returns which [drawing mode][DrawMode] this mesh should be drawn with.
    pub const fn draw_mode(&self) -> DrawMode {
        match self {
            Mesh::Triangle(_) | Mesh::IndexedTriangle(_) => DrawMode::Triangles,
            Mesh::Fan(_) | Mesh::IndexedFan(_) | Mesh::RestartIndexedFan(_) => DrawMode::TriangleFan,
            Mesh::Strip(_) | Mesh::IndexedStrip(_) | Mesh::RestartIndexedStrip(_) => DrawMode::TriangleStrip,
        }
    }
}


/// A mesh constructed of nothing more than a list of vertices.
pub struct BasicMesh<V: VertexData> {
    vertices: Vec<V>,
}

impl<V: VertexData> BasicMesh<V> {
    /// Creates a new basic mesh from a list of vertices.
    pub fn new(vertex_list: impl Into<Vec<V>>) -> Self {
        Self { vertices: vertex_list.into() }
    }
}


/// A mesh constructed of a list of vertex data alongside a list of indices into that data.
pub struct IndexedMesh<V: VertexData> {
    vertices: Vec<V>,
    indices: Vec<GLuint>,
}

impl<V: VertexData> IndexedMesh<V> {
    /// Creates a new indexed mesh from some vertex data and a list of indices.
    pub fn new(vertex_data: impl Into<Vec<V>>, indices: impl Into<Vec<GLuint>>) -> Self {
        Self {
            vertices: vertex_data.into(),
            indices: indices.into(),
        }
    }
}


/// A regular [indexed mesh][IndexedMesh], but one with an additional _primitive restart index;_ a sentinel value used
/// to tell OpenGL to split up one large buffer into multiple primitives of the same type.
pub struct RestartIndexedMesh<V: VertexData> {
    /// Which value in the index buffer should be used for the _primitive restart index._
    pub restart_idx: GLuint,
    pub mesh: IndexedMesh<V>,
}

impl<V: VertexData> RestartIndexedMesh<V> {
    /// Constructs a new indexed mesh with a _primitive restart index_ for restarting primitives.
    pub fn new(vertex_data: impl Into<Vec<V>>, indices: impl Into<Vec<GLuint>>, restart_idx: GLuint) -> Self {
        Self {
            restart_idx,
            mesh: IndexedMesh::new(vertex_data, indices),
        }
    }
}

use super::macros::gl_enum;


gl_enum! {
    /// A type of shader used in an OpenGL program.
    pub enum ShaderType {
        Compute => COMPUTE_SHADER,
        Fragment => FRAGMENT_SHADER,
        Geometry => GEOMETRY_SHADER,
        TessellationControl => TESS_CONTROL_SHADER,
        TessellationEvaluation => TESS_EVALUATION_SHADER,
        Vertex => VERTEX_SHADER,
    }
}


gl_enum! {
    /// Patterns for buffer data stores.
    ///
    /// Each value can be broken down into two parts: the frequency of access and the nature of access. The frequency of
    /// access may be one of:
    ///
    /// - **`Stream:`** the data store contents will be modified once and used at most a few times.
    /// - **`Static:`** the data store contents will be modified once and used many times.
    /// - **`Dynamic:`** the data store contents will be modified repeatedly and used many times.
    ///
    /// And the nature of access may be one of:
    ///
    /// - **`Draw:`** the data store contents are modified by the application, and used as the source for GL drawing and
    ///   image specification commands.
    /// - **`Read:`** the data store contents are modified by reading data from the GL, and used to return that data when
    ///   queried by the application.
    /// - **`Copy:`** the data store contents are modified by reading data from the GL, and used as the source for GL
    ///   drawing and image specification commands.
    ///
    /// For example, [`StaticDraw`][Self::StaticDraw], which one might use for vertex data, means that the buffer's data
    /// store will be written into only once, and that it should be used as a source for GL drawing.
    pub enum BufferUsage {
        StreamDraw => STREAM_DRAW,
        StreamRead => STREAM_READ,
        StreamCopy => STREAM_COPY,
        StaticDraw => STATIC_DRAW,
        StaticRead => STATIC_READ,
        StaticCopy => STATIC_COPY,
        DynamicDraw => DYNAMIC_DRAW,
        DynamicRead => DYNAMIC_READ,
        DynamicCopy => DYNAMIC_COPY,
    }
}


gl_enum! {
    /// Buffer binding targets.
    pub enum BufferTarget {
        /// Buffer target for vertex attributes.
        ArrayBuffer => ARRAY_BUFFER,

        /// Buffer target for atomic counter storage.
        AtomicCounterBuffer => ATOMIC_COUNTER_BUFFER,

        /// Buffer target for the source of buffer copies.
        CopyReadBuffer => COPY_READ_BUFFER,

        /// Buffer target for the destination of buffer copies.
        CopyWriteBuffer => COPY_WRITE_BUFFER,

        /// Buffer target for indirect compute dispatch commands.
        DispatchIndirectBuffer => DISPATCH_INDIRECT_BUFFER,

        /// Buffer target for indirect command arguments.
        DrawIndirectBuffer => DRAW_INDIRECT_BUFFER,

        /// Buffer target for vertex array indices.
        ElementArrayBuffer => ELEMENT_ARRAY_BUFFER,

        /// Buffer target for the destination of pixel read operations.
        PixelPackBuffer => PIXEL_PACK_BUFFER,

        /// Buffer target for the source of texture data.
        PixelUnpackBuffer => PIXEL_UNPACK_BUFFER,

        /// Buffer target for the query results.
        QueryBuffer => QUERY_BUFFER,

        /// Buffer target for read-write storage for shaders.
        ShaderStorageBuffer => SHADER_STORAGE_BUFFER,

        /// Buffer target for texture data.
        TextureBuffer => TEXTURE_BUFFER,

        /// Buffer target for transform feedback data.
        TransformFeedbackBuffer => TRANSFORM_FEEDBACK_BUFFER,

        /// Buffer target for uniform block storage.
        UniformBuffer => UNIFORM_BUFFER,
    }
}


gl_enum! {
    pub enum VertexAttribType {
        Float => FLOAT,
        HalfFloat => HALF_FLOAT,
        Double => DOUBLE,
        Fixed => FIXED,

        Byte => BYTE,
        UnsignedByte => UNSIGNED_BYTE,
        Short => SHORT,
        UnsignedShort => UNSIGNED_SHORT,
        Int => INT,
        UnsignedInt => UNSIGNED_INT,

        SignedIntFourPack => INT_2_10_10_10_REV,
        UnsignedIntFourPack => UNSIGNED_INT_2_10_10_10_REV,
        FloatThreePack => UNSIGNED_INT_10F_11F_11F_REV,
    }
}


gl_enum! {
    pub enum IntegerVertexAttribType {
        Byte => BYTE,
        UnsignedByte => UNSIGNED_BYTE,
        Short => SHORT,
        UnsignedShort => UNSIGNED_SHORT,
        Int => INT,
        UnsignedInt => UNSIGNED_INT,
    }
}

impl From<IntegerVertexAttribType> for VertexAttribType {
    fn from(value: IntegerVertexAttribType) -> Self {
        match value {
            IntegerVertexAttribType::Byte => Self::Byte,
            IntegerVertexAttribType::UnsignedByte => Self::UnsignedByte,
            IntegerVertexAttribType::Short => Self::Short,
            IntegerVertexAttribType::UnsignedShort => Self::UnsignedShort,
            IntegerVertexAttribType::Int => Self::Int,
            IntegerVertexAttribType::UnsignedInt => Self::UnsignedInt,
        }
    }
}


gl_enum! {
    pub enum DoubleVertexAttribType {
        Double => DOUBLE,
    }
}

impl From<DoubleVertexAttribType> for VertexAttribType {
    fn from(value: DoubleVertexAttribType) -> Self {
        match value {
            DoubleVertexAttribType::Double => Self::Double,
        }
    }
}


gl_enum! {
    pub enum DrawMode {
        Points => POINTS,
        LineStrip => LINE_STRIP,
        LineLoop => LINE_LOOP,
        Lines => LINES,
        LineStripAdjacency => LINE_STRIP_ADJACENCY,
        LinesAdjacency => LINES_ADJACENCY,
        TriangleStrip => TRIANGLE_STRIP,
        TriangleFan => TRIANGLE_FAN,
        Triangles => TRIANGLES,
        TriangleStripAdjacency => TRIANGLE_STRIP_ADJACENCY,
        TrianglesAdjacency => TRIANGLES_ADJACENCY,
        Patches => PATCHES,
    }
}

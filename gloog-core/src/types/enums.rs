use crate::macros::gl_enum;


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
    pub enum DrawElementsType {
        UnsignedByte => UNSIGNED_BYTE,
        UnsignedShort => UNSIGNED_SHORT,
        UnsignedInt => UNSIGNED_INT,
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


gl_enum! {
    pub enum EnableCap {
        /// If enabled, blend the computed fragment color values with the values in the color buffers. See
        /// `glBlendFunc`.
        Blend => BLEND,

        /// If enabled, clip geometry against user-defined half space 0.
        ClipDistance0 => CLIP_DISTANCE0,
        /// If enabled, clip geometry against user-defined half space 1.
        ClipDistance1 => CLIP_DISTANCE1,
        /// If enabled, clip geometry against user-defined half space 2.
        ClipDistance2 => CLIP_DISTANCE2,
        /// If enabled, clip geometry against user-defined half space 3.
        ClipDistance3 => CLIP_DISTANCE3,
        /// If enabled, clip geometry against user-defined half space 4.
        ClipDistance4 => CLIP_DISTANCE4,
        /// If enabled, clip geometry against user-defined half space 5.
        ClipDistance5 => CLIP_DISTANCE5,
        /// If enabled, clip geometry against user-defined half space 6.
        ClipDistance6 => CLIP_DISTANCE6,
        /// If enabled, clip geometry against user-defined half space 7.
        ClipDistance7 => CLIP_DISTANCE7,

        /// If enabled, apply the currently selected logical operation to the computed fragment color and color buffer
        /// values. See `glLogicOp`.
        ColorLogicOp => COLOR_LOGIC_OP,

        /// If enabled, cull polygons based on their winding in window coordinates. See `glCullFace`.
        CullFace => CULL_FACE,

        /// If enabled, debug messages are produced by a debug context. When disabled, the debug message log is
        /// silenced. Note that in a non-debug context, very few, if any messages might be produced, even when
        /// `GL_DEBUG_OUTPUT` is enabled.
        DebugOutput => DEBUG_OUTPUT,

        /// If enabled, debug messages are produced synchronously by a debug context. If disabled, debug messages may be
        /// produced asynchronously. In particular, they may be delayed relative to the execution of GL commands, and
        /// the debug callback function may be called from a thread other than that in which the commands are executed.
        /// See `glDebugMessageCallback`.
        DebugOutputSynchronous => DEBUG_OUTPUT_SYNCHRONOUS,

        /// If enabled, the `-wc ≤ zc ≤ wc` plane equation is ignored by view volume clipping (effectively, there is no
        /// near or far plane clipping). See `glDepthRange`.
        DepthClamp => DEPTH_CLAMP,

        /// If enabled, do depth comparisons and update the depth buffer. Note that even if the depth buffer exists and
        /// the depth mask is non-zero, the depth buffer is not updated if the depth test is disabled. See `glDepthFunc`
        /// and `glDepthRange`.
        DepthTest => DEPTH_TEST,

        /// If enabled, dither color components or indices before they are written to the color buffer.
        Dither => DITHER,

        /// If enabled and the value of `GL_FRAMEBUFFER_ATTACHMENT_COLOR_ENCODING` for the framebuffer attachment
        /// corresponding to the destination buffer is `GL_SRGB`, the R, G, and B destination color values (after
        /// conversion from fixed-point to floating-point) are considered to be encoded for the sRGB color space and
        /// hence are linearized prior to their use in blending.
        FramebufferSRGB => FRAMEBUFFER_SRGB,

        /// If enabled, draw lines with correct filtering. Otherwise, draw aliased lines. See `glLineWidth`.
        LineSmooth => LINE_SMOOTH,

        /// If enabled, use multiple fragment samples in computing the final color of a pixel. See `glSampleCoverage`.
        Multisample => MULTISAMPLE,

        /// If enabled, and if the polygon is rendered in `GL_FILL` mode, an offset is added to depth values of a
        /// polygon's fragments before the depth comparison is performed. See `glPolygonOffset`.
        PolygonOffsetFill => POLYGON_OFFSET_FILL,

        /// If enabled, and if the polygon is rendered in `GL_LINE` mode, an offset is added to depth values of a
        /// polygon's fragments before the depth comparison is performed. See `glPolygonOffset`.
        PolygonOffsetLine => POLYGON_OFFSET_LINE,

        /// If enabled, an offset is added to depth values of a polygon's fragments before the depth comparison is
        /// performed, if the polygon is rendered in `GL_POINT` mode. See `glPolygonOffset`.
        PolygonOffsetPoint => POLYGON_OFFSET_POINT,

        /// If enabled, draw polygons with proper filtering. Otherwise, draw aliased polygons. For correct antialiased
        /// polygons, an alpha buffer is needed and the polygons must be sorted front to back.
        PolygonSmooth => POLYGON_SMOOTH,

        /// Enables primitive restarting. If enabled, any one of the draw commands which transfers a set of generic
        /// attribute array elements to the GL will restart the primitive when the index of the vertex is equal to the
        /// primitive restart index. See `glPrimitiveRestartIndex`.
        PrimitiveRestart => PRIMITIVE_RESTART,

        /// Enables primitive restarting with a fixed index. If enabled, any one of the draw commands which transfers a
        /// set of generic attribute array elements to the GL will restart the primitive when the index of the vertex is
        /// equal to the fixed primitive index for the specified index type. The fixed index is equal to `2n-1` where
        /// `n` is equal to 8 for `GL_UNSIGNED_BYTE`, 16 for `GL_UNSIGNED_SHORT` and 32 for `GL_UNSIGNED_INT`.
        PrimitiveRestartFixedIndex => PRIMITIVE_RESTART_FIXED_INDEX,

        /// If enabled, primitives are discarded after the optional transform feedback stage, but before rasterization.
        /// Furthermore, when enabled, `glClear`, `glClearBufferData`, `glClearBufferSubData`, `glClearTexImage`, and
        /// `glClearTexSubImage` are ignored.
        RasterizerDiscard => RASTERIZER_DISCARD,

        /// If enabled, compute a temporary coverage value where each bit is determined by the alpha value at the
        /// corresponding sample location. The temporary coverage value is then ANDed with the fragment coverage value.
        SampleAlphaToCoverage => SAMPLE_ALPHA_TO_COVERAGE,

        /// If enabled, each sample alpha value is replaced by the maximum representable alpha value.
        SampleAlphaToOne => SAMPLE_ALPHA_TO_ONE,

        /// If enabled, the fragment's coverage is ANDed with the temporary coverage value. If
        /// `GL_SAMPLE_COVERAGE_INVERT` is set to `GL_TRUE`, invert the coverage value. See `glSampleCoverage`.
        SampleCoverage => SAMPLE_COVERAGE,

        /// If enabled, the active fragment shader is run once for each covered sample, or at fraction of this rate as
        /// determined by the current value of `GL_MIN_SAMPLE_SHADING_VALUE`. See `glMinSampleShading`.
        SampleShading => SAMPLE_SHADING,

        /// If enabled, the sample coverage mask generated for a fragment during rasterization will be ANDed with the
        /// value of `GL_SAMPLE_MASK_VALUE` before shading occurs. See `glSampleMaski`.
        SampleMask => SAMPLE_MASK,

        /// If enabled, discard fragments that are outside the scissor rectangle. See `glScissor`.
        ScissorTest => SCISSOR_TEST,

        /// If enabled, do stencil testing and update the stencil buffer. See `glStencilFunc` and `glStencilOp`.
        StencilTest => STENCIL_TEST,

        /// If enabled, cubemap textures are sampled such that when linearly sampling from the border between two
        /// adjacent faces, texels from both faces are used to generate the final sample value. When disabled, texels
        /// from only a single face are used to construct the final sample value.
        TextureCubeMapSeamless => TEXTURE_CUBE_MAP_SEAMLESS,

        /// If enabled and a vertex or geometry shader is active, then the derived point size is taken from the
        /// (potentially clipped) shader builtin `gl_PointSize` and clamped to the implementation-dependent point size
        /// range.
        ProgramPointSize => PROGRAM_POINT_SIZE,
    }
}

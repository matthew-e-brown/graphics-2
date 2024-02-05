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
        // /// If enabled, do alpha testing. See glAlphaFunc.
        // AlphaTest => ALPHA_TEST,

        // /// If enabled, generate normal vectors when either GL_MAP2_VERTEX_3 or GL_MAP2_VERTEX_4 is used to generate
        // /// vertices. See glMap2.
        // AutoNormal => AUTO_NORMAL,

        /// If enabled, blend the computed fragment color values with the values in the color buffers. See glBlendFunc.
        Blend => BLEND,

        // /// If enabled, clip geometry against user-defined clipping plane i. See glClipPlane.
        // ClipPlane0 => CLIP_PLANE0,

        // /// If enabled, clip geometry against user-defined clipping plane i. See glClipPlane.
        // ClipPlane1 => CLIP_PLANE1,

        // /// If enabled, clip geometry against user-defined clipping plane i. See glClipPlane.
        // ClipPlane2 => CLIP_PLANE2,

        // /// If enabled, clip geometry against user-defined clipping plane i. See glClipPlane.
        // ClipPlane3 => CLIP_PLANE3,

        // /// If enabled, clip geometry against user-defined clipping plane i. See glClipPlane.
        // ClipPlane4 => CLIP_PLANE4,

        // /// If enabled, clip geometry against user-defined clipping plane i. See glClipPlane.
        // ClipPlane5 => CLIP_PLANE5,

        /// If enabled, apply the currently selected logical operation to the computed fragment color and color buffer
        /// values. See glLogicOp.
        ColorLogicOp => COLOR_LOGIC_OP,

        // /// If enabled, have one or more material parameters track the current color. See glColorMaterial.
        // ColorMaterial => COLOR_MATERIAL,

        // /// If enabled and no fragment shader is active, add the secondary color value to the computed fragment color.
        // /// See glSecondaryColor.
        // ColorSum => COLOR_SUM,

        // /// If enabled, perform a color table lookup on the incoming RGBA color values. See glColorTable.
        // ColorTable => COLOR_TABLE,

        // /// If enabled, perform a 1D convolution operation on incoming RGBA color values. See glConvolutionFilter1D.
        // Convolution1d => CONVOLUTION_1D,

        // /// If enabled, perform a 2D convolution operation on incoming RGBA color values. See glConvolutionFilter2D.
        // Convolution2d => CONVOLUTION_2D,

        /// If enabled, cull polygons based on their winding in window coordinates. See glCullFace.
        CullFace => CULL_FACE,

        /// If enabled, do depth comparisons and update the depth buffer. Note that even if the depth buffer exists and
        /// the depth mask is non-zero, the depth buffer is not updated if the depth test is disabled. See glDepthFunc
        /// and glDepthRange.
        DepthTest => DEPTH_TEST,

        /// If enabled, dither color components or indices before they are written to the color buffer.
        Dither => DITHER,

        // /// If enabled and no fragment shader is active, blend a fog color into the post-texturing color. See glFog.
        // Fog => FOG,

        // /// If enabled, histogram incoming RGBA color values. See glHistogram.
        // Histogram => HISTOGRAM,

        // /// If enabled, apply the currently selected logical operation to the incoming index and color buffer indices.
        // /// See glLogicOp.
        // IndexLogicOp => INDEX_LOGIC_OP,

        // /// If enabled, include light i in the evaluation of the lighting equation. See glLightModel and glLight.
        // Light0 => LIGHT0,

        // /// If enabled, include light i in the evaluation of the lighting equation. See glLightModel and glLight.
        // Light1 => LIGHT1,

        // /// If enabled, include light i in the evaluation of the lighting equation. See glLightModel and glLight.
        // Light2 => LIGHT2,

        // /// If enabled, include light i in the evaluation of the lighting equation. See glLightModel and glLight.
        // Light3 => LIGHT3,

        // /// If enabled, include light i in the evaluation of the lighting equation. See glLightModel and glLight.
        // Light4 => LIGHT4,

        // /// If enabled, include light i in the evaluation of the lighting equation. See glLightModel and glLight.
        // Light5 => LIGHT5,

        // /// If enabled, include light i in the evaluation of the lighting equation. See glLightModel and glLight.
        // Light6 => LIGHT6,

        // /// If enabled, include light i in the evaluation of the lighting equation. See glLightModel and glLight.
        // Light7 => LIGHT7,

        // /// If enabled and no vertex shader is active, use the current lighting parameters to compute the vertex color
        // /// or index. Otherwise, simply associate the current color or index with each vertex. See glMaterial,
        // /// glLightModel, and glLight.
        // Lighting => LIGHTING,

        /// If enabled, draw lines with correct filtering. Otherwise, draw aliased lines. See glLineWidth.
        LineSmooth => LINE_SMOOTH,

        // /// If enabled, use the current line stipple pattern when drawing lines. See glLineStipple.
        // LineStipple => LINE_STIPPLE,

        // /// If enabled, calls to glEvalCoord1, glEvalMesh1, and glEvalPoint1 generate RGBA values. See glMap1.
        // Map1Color4 => MAP1_COLOR_4,

        // /// If enabled, calls to glEvalCoord1, glEvalMesh1, and glEvalPoint1 generate color indices. See glMap1.
        // Map1Index => MAP1_INDEX,

        // /// If enabled, calls to glEvalCoord1, glEvalMesh1, and glEvalPoint1 generate normals. See glMap1.
        // Map1Normal => MAP1_NORMAL,

        // /// If enabled, calls to glEvalCoord1, glEvalMesh1, and glEvalPoint1 generate s texture coordinates. See glMap1.
        // Map1TextureCoord1 => MAP1_TEXTURE_COORD_1,

        // /// If enabled, calls to glEvalCoord1, glEvalMesh1, and glEvalPoint1 generate s and t texture coordinates. See
        // /// glMap1.
        // Map1TextureCoord2 => MAP1_TEXTURE_COORD_2,

        // /// If enabled, calls to glEvalCoord1, glEvalMesh1, and glEvalPoint1 generate s, t, and r texture coordinates.
        // /// See glMap1.
        // Map1TextureCoord3 => MAP1_TEXTURE_COORD_3,

        // /// If enabled, calls to glEvalCoord1, glEvalMesh1, and glEvalPoint1 generate s, t, r, and q texture
        // /// coordinates. See glMap1.
        // Map1TextureCoord4 => MAP1_TEXTURE_COORD_4,

        // /// If enabled, calls to glEvalCoord1, glEvalMesh1, and glEvalPoint1 generate x, y, and z vertex coordinates.
        // /// See glMap1.
        // Map1Vertex3 => MAP1_VERTEX_3,

        // /// If enabled, calls to glEvalCoord1, glEvalMesh1, and glEvalPoint1 generate homogeneous x, y, z, and w vertex
        // /// coordinates. See glMap1.
        // Map1Vertex4 => MAP1_VERTEX_4,

        // /// If enabled, calls to glEvalCoord2, glEvalMesh2, and glEvalPoint2 generate RGBA values. See glMap2.
        // Map2Color4 => MAP2_COLOR_4,

        // /// If enabled, calls to glEvalCoord2, glEvalMesh2, and glEvalPoint2 generate color indices. See glMap2.
        // Map2Index => MAP2_INDEX,

        // /// If enabled, calls to glEvalCoord2, glEvalMesh2, and glEvalPoint2 generate normals. See glMap2.
        // Map2Normal => MAP2_NORMAL,

        // /// If enabled, calls to glEvalCoord2, glEvalMesh2, and glEvalPoint2 generate s texture coordinates. See glMap2.
        // Map2TextureCoord1 => MAP2_TEXTURE_COORD_1,

        // /// If enabled, calls to glEvalCoord2, glEvalMesh2, and glEvalPoint2 generate s and t texture coordinates. See
        // /// glMap2.
        // Map2TextureCoord2 => MAP2_TEXTURE_COORD_2,

        // /// If enabled, calls to glEvalCoord2, glEvalMesh2, and glEvalPoint2 generate s, t, and r texture coordinates.
        // /// See glMap2.
        // Map2TextureCoord3 => MAP2_TEXTURE_COORD_3,

        // /// If enabled, calls to glEvalCoord2, glEvalMesh2, and glEvalPoint2 generate s, t, r, and q texture
        // /// coordinates. See glMap2.
        // Map2TextureCoord4 => MAP2_TEXTURE_COORD_4,

        // /// If enabled, calls to glEvalCoord2, glEvalMesh2, and glEvalPoint2 generate x, y, and z vertex coordinates.
        // /// See glMap2.
        // Map2Vertex3 => MAP2_VERTEX_3,

        // /// If enabled, calls to glEvalCoord2, glEvalMesh2, and glEvalPoint2 generate homogeneous x, y, z, and w vertex
        // /// coordinates. See glMap2.
        // Map2Vertex4 => MAP2_VERTEX_4,

        // /// If enabled, compute the minimum and maximum values of incoming RGBA color values. See glMinmax.
        // Minmax => MINMAX,

        // /// If enabled, use multiple fragment samples in computing the final color of a pixel. See glSampleCoverage.
        // Multisample => MULTISAMPLE,

        // /// If enabled and no vertex shader is active, normal vectors are normalized to unit length after transformation
        // /// and before lighting. This method is generally less efficient than GL_RESCALE_NORMAL. See glNormal and
        // /// glNormalPointer.
        // Normalize => NORMALIZE,

        // /// If enabled, draw points with proper filtering. Otherwise, draw aliased points. See glPointSize.
        // PointSmooth => POINT_SMOOTH,

        // /// If enabled, calculate texture coordinates for points based on texture environment and point parameter
        // /// settings. Otherwise texture coordinates are constant across points.
        // PointSprite => POINT_SPRITE,

        /// If enabled, and if the polygon is rendered in GL_FILL mode, an offset is added to depth values of a
        /// polygon's fragments before the depth comparison is performed. See glPolygonOffset.
        PolygonOffsetFill => POLYGON_OFFSET_FILL,

        /// If enabled, and if the polygon is rendered in GL_LINE mode, an offset is added to depth values of a
        /// polygon's fragments before the depth comparison is performed. See glPolygonOffset.
        PolygonOffsetLine => POLYGON_OFFSET_LINE,

        /// If enabled, an offset is added to depth values of a polygon's fragments before the depth comparison is
        /// performed, if the polygon is rendered in GL_POINT mode. See glPolygonOffset.
        PolygonOffsetPoint => POLYGON_OFFSET_POINT,

        /// If enabled, draw polygons with proper filtering. Otherwise, draw aliased polygons. For correct antialiased
        /// polygons, an alpha buffer is needed and the polygons must be sorted front to back.
        PolygonSmooth => POLYGON_SMOOTH,

        // /// If enabled, use the current polygon stipple pattern when rendering polygons. See glPolygonStipple.
        // PolygonStipple => POLYGON_STIPPLE,

        // /// If enabled, perform a color table lookup on RGBA color values after color matrix transformation. See
        // /// glColorTable.
        // PostColorMatrixColorTable => POST_COLOR_MATRIX_COLOR_TABLE,

        // /// If enabled, perform a color table lookup on RGBA color values after convolution. See glColorTable.
        // PostConvolutionColorTable => POST_CONVOLUTION_COLOR_TABLE,

        // /// If enabled and no vertex shader is active, normal vectors are scaled after transformation and before
        // /// lighting by a factor computed from the model-view matrix. If the model-view matrix scales space uniformly,
        // /// this has the effect of restoring the transformed normal to unit length. This method is generally more
        // /// efficient than GL_NORMALIZE. See glNormal and glNormalPointer.
        // RescaleNormal => RESCALE_NORMAL,

        // /// If enabled, compute a temporary coverage value where each bit is determined by the alpha value at the
        // /// corresponding sample location. The temporary coverage value is then ANDed with the fragment coverage value.
        // SampleAlphaToCoverage => SAMPLE_ALPHA_TO_COVERAGE,

        // /// If enabled, each sample alpha value is replaced by the maximum representable alpha value.
        // SampleAlphaToOne => SAMPLE_ALPHA_TO_ONE,

        // /// If enabled, the fragment's coverage is ANDed with the temporary coverage value. If GL_SAMPLE_COVERAGE_INVERT
        // /// is set to GL_TRUE, invert the coverage value. See glSampleCoverage.
        // SampleCoverage => SAMPLE_COVERAGE,

        // /// If enabled, perform a two-dimensional convolution operation using a separable convolution filter on incoming
        // /// RGBA color values. See glSeparableFilter2D.
        // Separable2d => SEPARABLE_2D,

        /// If enabled, discard fragments that are outside the scissor rectangle. See glScissor.
        ScissorTest => SCISSOR_TEST,

        /// If enabled, do stencil testing and update the stencil buffer. See glStencilFunc and glStencilOp.
        StencilTest => STENCIL_TEST,

        /// If enabled and no fragment shader is active, one-dimensional texturing is performed (unless two- or
        /// three-dimensional or cube-mapped texturing is also enabled). See glTexImage1D.
        Texture1d => TEXTURE_1D,

        /// If enabled and no fragment shader is active, two-dimensional texturing is performed (unless
        /// three-dimensional or cube-mapped texturing is also enabled). See glTexImage2D.
        Texture2d => TEXTURE_2D,

        /// If enabled and no fragment shader is active, three-dimensional texturing is performed (unless cube-mapped
        /// texturing is also enabled). See glTexImage3D.
        Texture3d => TEXTURE_3D,

        /// If enabled and no fragment shader is active, cube-mapped texturing is performed. See glTexImage2D.
        TextureCubeMap => TEXTURE_CUBE_MAP,

        // /// If enabled and no vertex shader is active, the q texture coordinate is computed using the texture generation
        // /// function defined with glTexGen. Otherwise, the current q texture coordinate is used. See glTexGen.
        // TextureGenQ => TEXTURE_GEN_Q,

        // /// If enabled and no vertex shader is active, the r texture coordinate is computed using the texture generation
        // /// function defined with glTexGen. Otherwise, the current r texture coordinate is used. See glTexGen.
        // TextureGenR => TEXTURE_GEN_R,

        // /// If enabled and no vertex shader is active, the s texture coordinate is computed using the texture generation
        // /// function defined with glTexGen. Otherwise, the current s texture coordinate is used. See glTexGen.
        // TextureGenS => TEXTURE_GEN_S,

        // /// If enabled and no vertex shader is active, the t texture coordinate is computed using the texture generation
        // /// function defined with glTexGen. Otherwise, the current t texture coordinate is used. See glTexGen.
        // TextureGenT => TEXTURE_GEN_T,

        // /// If enabled and a vertex shader is active, then the derived point size is taken from the (potentially
        // /// clipped) shader builtin gl_PointSize and clamped to the implementation-dependent point size range.
        // VertexProgramPointSize => VERTEX_PROGRAM_POINT_SIZE,

        // /// If enabled and a vertex shader is active, it specifies that the GL will choose between front and back colors
        // /// based on the polygon's face direction of which the vertex being shaded is a part. It has no effect on points
        // /// or lines.
        // VertexProgramTwoSide => VERTEX_PROGRAM_TWO_SIDE,
    }
}

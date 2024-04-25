//! Types and implementations related to "meta" commands such as state queries, capability enablement, and so on.

use std::ffi::CStr;

use crate::raw::types::*;
use crate::{gl_enum, GLContext};

gl_enum! {
    /// Capabilities which may be enabled or disabled with [`GLContext::enable`] and [`GLContext::disable`].
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

gl_enum! {
    /// Which vertex winding direction counts as "front-facing". For use with [`GLContext::front_face`].
    pub enum FrontFaceDirection {
        CW => CW,
        CCW => CCW,
    }
}

gl_enum! {
    // This enum group is called `TriangleFace` in gl.xml; since it is used by `glPolygonMode` (and others, but none of
    // them specify triangles vs. polygons), we call it `PolygonFace` instead.
    pub enum PolygonFace {
        Back => BACK,
        Front => FRONT,
        FrontAndBack => FRONT_AND_BACK,
    }
}

gl_enum! {
    /// Different rasterization modes for polygons. For use with [`GLContext::polygon_mode`].
    pub enum PolygonMode {
        /// Causes the vertices of a polygon to be treated, for rasterization purposes, as if it had been drawn with
        /// _mode_ `POINTS`.
        Point => POINT,
        /// Causes edges to be rasterized as line segments.
        Line => LINE,
        /// The default mode of polygon rasterization.
        Fill => FILL,
    }
}

gl_enum! {
    pub enum StringName {
        Renderer => RENDERER,
        Vendor => VENDOR,
        Version => VERSION,
        ShadingLanguageVersion => SHADING_LANGUAGE_VERSION,
    }
}

gl_enum! {
    pub enum IndexedStringName {
        Extensions => EXTENSIONS,
        ShadingLanguageVersion => SHADING_LANGUAGE_VERSION,
        SPIRVExtensions => SPIR_V_EXTENSIONS,
    }
}


impl GLContext {
    /// Enable capacities in OpenGL.
    ///
    /// Refer to [`EnableCap`] for a list of possible capabilities.
    pub fn enable(&mut self, cap: EnableCap) {
        unsafe { self.gl.enable(cap.into_raw()) }
    }

    /// Disable capacities in OpenGL.
    ///
    /// Refer to [`EnableCap`] for a list of possible capabilities.
    pub fn disable(&mut self, cap: EnableCap) {
        unsafe { self.gl.disable(cap.into_raw()) }
    }

    // [TODO] Implement `glGet` queries for simple values; there are a _ton_ of possible values for `glInteger_v`,
    // `glBoolean_v`, etc. Some return a single value, some return multiple. Probably gonna be a pain to implement.

    /// Queries this context's state for a string value.
    pub fn get_string(&self, name: StringName) -> String {
        let ptr = unsafe { self.gl.get_string(name.into_raw()) };
        // SAFETY: OpenGL 4.6 core spec (p.566/588, section 22.2) says that "string queries return pointers to UTF-8
        // encoded, null-terminated static strings describing properties of the current GL context." So this pointer
        // should always be valid, unless an implementation provides a bogus one.
        let str = unsafe { CStr::from_ptr(ptr.cast()) };
        str.to_string_lossy().into_owned()
    }

    /// Queries this context's state for an indexed string value.
    pub fn get_string_i(&self, name: IndexedStringName, index: u32) -> String {
        let ptr = unsafe { self.gl.get_string_i(name.into_raw(), index) };
        // SAFETY: See `get_string`.
        let str = unsafe { CStr::from_ptr(ptr.cast()) };
        str.to_string_lossy().into_owned()
    }

    /// Controls the facing direction of polygons.
    pub fn front_face(&mut self, dir: FrontFaceDirection) {
        unsafe { self.gl.front_face(dir.into_raw()) }
    }

    /// Controls which sides of a polygon are rasterized. "Front" and "back" are specified with
    /// [`GLContext::front_face`].
    ///
    /// Culling is disabled by default. It should be enabled by calling [`GLContext::enable`] with the target capacity
    /// of [`EnableCap::CullFace`].
    pub fn cull_face(&mut self, mode: PolygonFace) {
        unsafe { self.gl.cull_face(mode.into_raw()) }
    }

    /// Controls the interpretation of polygons for rasterization.
    pub fn polygon_mode(&mut self, mode: PolygonMode) {
        // From the docs on `glPolygonMode` (section 14.6.4), "face must be GL_FRONT_AND_BACK"
        const FACE: GLenum = PolygonFace::FrontAndBack.into_raw();
        unsafe { self.gl.polygon_mode(FACE, mode.into_raw()) }
    }
}

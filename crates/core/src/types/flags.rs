use super::macros::gl_bitfield;


gl_bitfield! {
    pub struct ClearMask {
        pub const COLOR = COLOR_BUFFER_BIT;
        pub const DEPTH = DEPTH_BUFFER_BIT;
        pub const STENCIL = STENCIL_BUFFER_BIT;
    }
}

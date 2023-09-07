/// Core OpenGL wrapper functions.
///
/// See the `gloog-core` crate for details.
pub use gloog_core as core;

/// Mathematic data structures and functions.
///
/// See the `gloog-math` crate for details.
pub mod math {
    // Re-export submodules of math directly under `math` name
    pub use gloog_math::matrix::*;
    pub use gloog_math::vector::*;
}

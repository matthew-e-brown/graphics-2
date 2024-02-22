//! Module for loading models from OBJ files, optionally paired with MTL material files.
//!
//! Only all-polygonal models are supported are the moment (no free-form surfaces). These are those that use only `f`,
//! no `curv` or `surf` statements. Unsupported statements are simply ignored, though a warning is produced.

mod error;
mod parsing;

// todo: move that stuff back out of `parsing`.

// Source for OBJ and MTL specs:
// - https://www.uhu.es/francisco.moreno/gii_rv/practicas/practica08/OBJ_SPEC.PDF
// - also: https://paulbourke.net/dataformats/obj/ (missing math section)
// - https://paulbourke.net/dataformats/mtl/

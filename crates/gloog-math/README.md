# Gloog &ndash; Math

Like *Gloog* itself, this crate is a helper library for my computer graphics
reading course. It provides helpful `Vec` and `Mat` structs for use with OpenGL.
Also like Gloog, its goal is to be as thin of a wrapper as possible: `Vec3` is
literally a transparent wrapper on top of `[f32; 3]` that provides helper
functions.

If you're curious about the name, refer to the main `gloog` crate's README.

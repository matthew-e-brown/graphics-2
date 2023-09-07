# Gloog &ndash; Core

This crate is the "core" of Gloog's rendering. The functions and types in this
crate directly wrap the bindings from the [`gl`][bindings] crate.

This crate's only goal is to "oxidize" the core OpenGL functions, globals, and
types. Things like RAII for OpenGL objects require keeping track of some extra
state, so that part is left out of the core crate. What this crate instead
allows for is using OpenGL functions from Rust without the need for `unsafe` or
casts all over the place. Additionally, that Gloog maintains compatibility with
OpenGL wikis, tutorials, and guides.

That aside, this crate's _primary_ purpose is to service my work for my computer
graphics reading course at university, which is my second-ish foray into OpenGL
(the first being in WebGL with JavaScript the year prior). There is a very real
possibility that this will never see any other use, and that's okay.

[bindings]: https://github.com/brendanzab/gl-rs

# Gloog &ndash; Bindgen

This crate is responsible for generating the bindings exported by _Gloog Core_.
It is implemented as a separate crate so that it can be documented, tested, and
updated separate from _Gloog Core_. This crate is inspired by
[`gl_generator`][gl-generator].

I'm writing my own crate for two reasons. Firstly, _Gloog_ is primarily a
learning exercise for my reading course; the more NIH, the better. Second,
`gl_generator` didn't quite have the features I wanted for generating my
bindings (namely that it didn't export `class` attributes on command
parameters). Rather than submitting and waiting for a pull request to be merged,
whipping my own up from scratch seemed like it would be more time-efficient (at
least for the specific subset of the spec I want to generate bindings for).


[gl-generator]: https://docs.rs/gl_generator/latest/gl_generator/

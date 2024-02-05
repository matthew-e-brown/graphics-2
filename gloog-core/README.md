# Gloog &ndash; Core

_Gloog_ is yet another graphics library for Rust. This "core" crate contains the
OpenGL bindings and wrappers that will be used for the larger library. The
larger library will include some extra helper functionality related to
initializing windows and such, most likely wrapping GLFW.


## Goals

The main goal for Gloog is to build something that provides low-overhead access
to OpenGL objects and functions with an API that is as oxidized as possible. It
is not going to be a _wrapper_ for OpenGL: it will be its own library _built on_
OpenGL. That means it will not support old versions of OpenGL, GLES, and so on
(for now).

Further goals:

- Use modern OpenGL DSA functions that act on objects directly to be able to
  provide safe abstractions over them (for example, a `Buffer` struct that takes
  a `&mut self` to update its data).
- Provide proper Rust enums for all parameters.
- Work using types defined in [`gloog-math`](../gloog-math/README.md)
- Some more stuff I'll remember when it isn't past midnight.

> [!NOTE]
>
> It should be noted that this crate's _primary_ purpose is to supplement my
> coursework for a computer graphics reading course. Its development is going to
> be immeasurably unstable and its code shall be gloriously opinionated.
>
> <small>(Additionally, this "goals" section is here more so to keep myself
> reminded and on-task, rather than to actually serving as some sort of
> manifesto. Without it, I'd keep trying to implement some huge, general purpose
> "OpenGL bindings for Rust" crate).</small>



## The name

The name *Gloog* is complete gibberish. It comes from when I was playing Civ 5
with my friends and needed [a funny name for my religion][religion]. Even though
that round was short-lived, the name stuck, and it became a silly placeholder
that I will use whenever a game we're playing needed me to pick a name for a
city, planet, species, or whatever else. Given its immense stupidity, my friends
all started to groan every time I named anything after "gloog". So, of course,
when the need to come up with an unused name came up... I thought it would be
funny to continue the legacy.

You can pretend that the first two letters of *gloog* stand for "graphics
library." *Graphics library of overwhelming greatness*, how's that sound? :grin:


[religion]: https://civilization.fandom.com/wiki/Religion_(Civ5)#Founding_a_Religion:~:text=or%20you%20can%20type%20a%20new%20name%20of%20your%20own.

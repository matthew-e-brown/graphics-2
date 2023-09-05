# Gloog

Gloog is a wrapper for OpenGL. It directly wraps the bindings from the
[`gl`][bindings] crate: its only goal is to "oxidize" the core OpenGL functions,
globals, and types so that they can be used from Rust applications without the
need for willy-nilly casts or `unsafe` everywhere. That way, it maintains
compatibility with OpenGL wikis, tutorials, and guides.

That said, this crate's **primary** purpose is to service my work for my
computer graphics reading course at university, which is my second-ish foray
into OpenGL (the first being in WebGL with JavaScript the year prior). There is
a very real possibility that this will never see any other use, and that's okay.


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


[bindings]: https://github.com/brendanzab/gl-rs
[religion]: https://civilization.fandom.com/wiki/Religion_(Civ5)#Founding_a_Religion:~:text=or%20you%20can%20type%20a%20new%20name%20of%20your%20own.

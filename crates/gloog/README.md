# Gloog

_Gloog_ is something between a graphics library and game engine for Rust, built
on top of OpenGL and GLFW.


## Crates

_Gloog_ re-exports things from a few smaller crates. These crates are separate
so that they may one day prove useful to others who are looking for some
graphics libraries, but perhaps don't need all of the functionality of Gloog.

- _Gloog Core_ has direct, one-to-one wrappers of raw OpenGL bindings.
- _Gloog Math_ has structures like matrices and vectors.


## The name

The name _Gloog_ is complete gibberish. It's a running joke I have with my
friends, which started during a game of _Sid Meier's Civilization 5_. Civ 5
[lets you pick names for your in-game religions][religion]; during one
particular match, after struggling to come up with anything funny for a while, I
just threw in a funny-sounding placeholder. Even though that match was
short-lived, the name stuck, that placeholder, Gloog, became a silly go-to name
that I still use whenever a game needs me to pick a name for a city, planet,
species, or whatever else you can imagine.

Given its immense stupidity, my friends all started to groan every time I named
anything after "gloog"&mdash;they were not happy when I started Gloog Transport,
Inc. in [OpenTTD][open-ttd].

So, naturally, when I suddenly needed to come up with an unused name for a
crate... I figured it only made sense to continue the legacy. If that story
doesn't site well with you, you can pretend that the first two letters of Gloog
stand for "graphics library" and go from there. Maybe it's short for _Graphics
Library of Overwhelming Greatness_, how's that? :grin:


[religion]: https://civilization.fandom.com/wiki/Religion_(Civ5)#Founding_a_Religion:~:text=or%20you%20can%20type%20a%20new%20name%20of%20your%20own.
[open-ttd]:https://www.openttd.org/

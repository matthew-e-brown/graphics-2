# Gloog &ndash; Math

This crate holds mathematical data structures and functions _Gloog_[^1].

At the moment, that's just matrices (`Mat2`, `Mat3`, and `Mat4`) and vectors
(`Vec2`, `Vec3`, and `Vec4`). In the future, `Quaternion`s will likely make an
appearance, as well as things like `Plane` and `Line` may be added with
additional methods/functionality.

This crate does its best to provide good operator overloading and conversion
support for these types.


[^1]: Gloog was extracted into [its own repository][gloog-repo]. Eventually,
    this crate will join it there; but for now, it is staying as part of this
    one.

[gloog-repo]: https://github.com/matthew-e-brown/gloog

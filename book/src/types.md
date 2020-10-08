# Types

`cxx` supports lots of types:

* Primitive types, e.g. `u32`.
* Built-in `cxx` types, [as listed here](https://docs.rs/cxx/0.4.7/cxx/#builtin-types).
* Structs declared in the `#[cxx::bridge]` mod, for which both C++ and Rust definitions are generated.
* Type aliases to other `#[cxx::bridge]` mods.
* Existing C++ types, which may be opaque or transparent to Rust. See [the chapter on C++ types for more details](cpp-types.md).
* Existing Rust types, which must by opaque to C++. See [the chapter on Rust types for more details](rust-types.md).
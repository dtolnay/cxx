# Performance

In general, the FFI bridge operates at zero or negligible overhead, i.e. no copying, no serialization, no memory allocation, no runtime checks needed.

The support for `&str` and `&[u8]` types - and their common use throughout good-quality Rust code - means that you may be able to avoid copies entirely. For instance, you can pass a string of JSON from C++ to Rust, parse it using [serde-json](https://github.com/serde-rs/json), and feed `&str` values back to C++ callbacks which will be received as `::rust::Str` types. Those `&str` locations will usually point back to spans of data in the _original_ C++ string, with zero copying involved at any time.

However, there are a few things to be aware of:

* *Rust strings are UTF-8; C++ strings are not*. You have full flexibility about whether to manipulate strings as [C++ strings](https://docs.rs/cxx/0.4.7/cxx/struct.CxxString.html) or Rust strings, but in many cases you will want to convert a C++ string to a Rust string somewhere. Plan ahead to minimize the impact of the UTF8 check. Consider if you can manipulate `&[u8]` slices instead of strings within the Rust code.
* *There are lots of fiddly little functions*. You may wish to enable [cross-language LTO](https://doc.rust-lang.org/rustc/linker-plugin-lto.html) such that they are inlined.

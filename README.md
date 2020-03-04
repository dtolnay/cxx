CXX &mdash; safe FFI between Rust and C++
=========================================

[![Build Status](https://api.travis-ci.com/dtolnay/cxx.svg?branch=master)](https://travis-ci.com/dtolnay/cxx)
[![Latest Version](https://img.shields.io/crates/v/cxx.svg)](https://crates.io/crates/cxx)
[![Rust Documentation](https://img.shields.io/badge/api-rustdoc-blue.svg)](https://docs.rs/cxx)

This library provides a **safe** mechanism for calling C++ code from Rust and
Rust code from C++, not subject to the many ways that things can go wrong when
using bindgen or cbindgen to generate unsafe C-style bindings.

This doesn't change the fact that 100% of C++ code is unsafe. When auditing a
project, you would be on the hook for auditing all the unsafe Rust code and
*all* the C++ code. The core safety claim under this new model is that auditing
just the C++ side would be sufficient to catch all problems, i.e. the Rust side
can be 100% safe.

```toml
[dependencies]
cxx = "0.1"
```

*Compiler support: requires rustc 1.42+ (beta on January 30, stable on March
12)*

<br>

## Overview

The idea is that we define the signatures of both sides of our FFI boundary
embedded together in one Rust module (the next section shows an example). From
this, CXX receives a complete picture of the boundary to perform static analyses
against the types and function signatures to uphold both Rust's and C++'s
invariants and requirements.

If everything checks out statically, then CXX uses a pair of code generators to
emit the relevant `extern "C"` signatures on both sides together with any
necessary static assertions for later in the build process to verify
correctness. On the Rust side this code generator is simply an attribute
procedural macro. On the C++ side it can be a small Cargo build script if your
build is managed by Cargo, or for other build systems like Bazel or Buck we
provide a command line tool which generates the header and source file and
should be easy to integrate.

The resulting FFI bridge operates at zero or negligible overhead, i.e. no
copying, no serialization, no memory allocation, no runtime checks needed.

The FFI signatures are able to use native types from whichever side they please,
such as Rust's `String` or C++'s `std::string`, Rust's `Box` or C++'s
`std::unique_ptr`, Rust's `Vec` or C++'s `std::vector`, etc in any combination.
CXX guarantees an ABI-compatible signature that both sides understand, based on
builtin bindings for key standard library types to expose an idiomatic API on
those types to the other language. For example when manipulating a C++ string
from Rust, its `len()` method becomes a call of the `size()` member function
defined by C++; when manipulation a Rust string from C++, its `size()` member
function calls Rust's `len()`.

<br>

## Example

A runnable version of this example is provided under the *demo-rs* directory of
this repo (with the C++ side of the implementation in the *demo-cxx* directory).
To try it out, jump into demo-rs and run `cargo run`.

```rust
#[cxx::bridge]
mod ffi {
    // Any shared structs, whose fields will be visible to both languages.
    struct SharedThing {
        z: i32,
        y: Box<ThingR>,
        x: UniquePtr<ThingC>,
    }

    extern "C" {
        // One or more headers with the matching C++ declarations. Our code
        // generators don't read it but it gets #include'd and used in static
        // assertions to ensure our picture of the FFI boundary is accurate.
        include!("demo-cxx/demo.h");

        // Zero or more opaque types which both languages can pass around but
        // only C++ can see the fields.
        type ThingC;

        // Functions implemented in C++.
        fn make_demo(appname: &str) -> UniquePtr<ThingC>;
        fn get_name(thing: &ThingC) -> &CxxString;
        fn do_thing(state: SharedThing);
    }

    extern "Rust" {
        // Zero or more opaque types which both languages can pass around but
        // only Rust can see the fields.
        type ThingR;

        // Functions implemented in Rust.
        fn print_r(r: &ThingR);
    }
}
```

Now we simply provide C++ definitions of all the things in the `extern "C"`
block and Rust definitions of all the things in the `extern "Rust"` block, and
get to call back and forth safely.

Here are links to the complete set of source files involved in the demo:

- [demo-rs/src/main.rs](demo-rs/src/main.rs)
- [demo-rs/build.rs](demo-rs/build.rs)
- [demo-cxx/demo.h](demo-cxx/demo.h)
- [demo-cxx/demo.cc](demo-cxx/demo.cc)

To look at the code generated in both languages for the example by the CXX code
generators:

```console
   # run Rust code generator and print to stdout
   # (requires https://github.com/dtolnay/cargo-expand)
$ cargo expand --manifest-path demo-rs/Cargo.toml

   # run C++ code generator and print to stdout
$ cargo run --manifest-path cmd/Cargo.toml -- demo-rs/src/main.rs
```

<br>

## Details

As seen in the example, the language of the FFI boundary involves 3 kinds of
items:

- **Shared structs** &mdash; their fields are made visible to both languages.
  The definition written within cxx::bridge is the single source of truth.

- **Opaque types** &mdash; their fields are secret from the other language.
  These cannot be passed across the FFI by value but only behind an indirection,
  such as a reference `&`, a Rust `Box`, or a `UniquePtr`. Can be a type alias
  for an arbitrarily complicated generic language-specific type depending on
  your use case.

- **Functions** &mdash; implemented in either language, callable from the other
  language.

Within the `extern "C"` part of the CXX bridge we list the types and functions
for which C++ is the source of truth, as well as the header(s) that declare
those APIs. In the future it's possible that this section could be generated
bindgen-style from the headers but for now we need the signatures written out;
static assertions will verify that they are accurate.

Within the `extern "Rust"` part, we list types and functions for which Rust is
the source of truth. These all implicitly refer to the `super` module, the
parent module of the CXX bridge. You can think of the two items listed in the
example above as being like `use super::ThingR` and `use super::print_r` except
re-exported to C++. The parent module will either contain the definitions
directly for simple things, or contain the relevant `use` statements to bring
them into scope from elsewhere.

Your function implementations themselves, whether in C++ or Rust, *do not* need
to be defined as `extern "C"` ABI or no\_mangle. CXX will put in the right shims
where necessary to make it all work.

<br>

## Comparison vs bindgen and cbindgen

Notice that with CXX there is repetition of all the function signatures: they
are typed out once where the implementation is defined (in C++ or Rust) and
again inside the cxx::bridge module, though compile-time assertions guarantee
these are kept in sync. This is different from [bindgen] and [cbindgen] where
function signatures are typed by a human once and the tool consumes them in one
language and emits them in the other language.

[bindgen]: https://github.com/rust-lang/rust-bindgen
[cbindgen]: https://github.com/eqrion/cbindgen/

This is because CXX fills a somewhat different role. It is a lower level tool
than bindgen or cbindgen in a sense; you can think of it as being a replacement
for the concept of `extern "C"` signatures as we know them, rather than a
replacement for a bindgen. It would be reasonable to build a higher level
bindgen-like tool on top of CXX which consumes a C++ header and/or Rust module
(and/or IDL like Thrift) as source of truth and generates the cxx::bridge,
eliminating the repetition while leveraging the static analysis safety
guarantees of CXX.

But note in other ways CXX is higher level than the bindgens, with rich support
for common standard library types. Frequently with bindgen when we are dealing
with an idiomatic C++ API we would end up manually wrapping that API in C-style
raw pointer functions, applying bindgen to get unsafe raw pointer Rust
functions, and replicating the API again to expose those idiomatically in Rust.
That's a much worse form of repetition because it is unsafe all the way through.

By using a CXX bridge as the shared understanding between the languages, rather
than `extern "C"` C-style signatures as the shared understanding, common FFI use
cases become expressible using 100% safe code.

It would also be reasonable to mix and match, using CXX bridge for the 95% of
your FFI that is straightforward and doing the remaining few oddball signatures
the old fashioned way with bindgen and cbindgen, if for some reason CXX's static
restrictions get in the way. Please file an issue if you end up taking this
approach so that we know what ways it would be worthwhile to make the tool more
expressive.

<br>

## Cargo-based setup

For builds that are orchestrated by Cargo, you will use a build script that runs
CXX's C++ code generator and compiles the resulting C++ code along with any
other C++ code for your crate.

The canonical build script is as follows. The indicated line returns a
[`cc::Build`] instance (from the usual widely used `cc` crate) on which you can
set up any additional source files and compiler flags as normal.

[`cc::Build`]: https://docs.rs/cc/1.0/cc/struct.Build.html

```rust
// build.rs

fn main() {
    cxx::Build::new()
        .bridge("src/main.rs")  // returns a cc::Build
        .file("../demo-cxx/demo.cc")
        .flag("-std=c++11")
        .compile("cxxbridge-demo");

    println!("cargo:rerun-if-changed=src/main.rs");
    println!("cargo:rerun-if-changed=../demo-cxx/demo.h");
    println!("cargo:rerun-if-changed=../demo-cxx/demo.cc");
}
```

<br>

## Non-Cargo setup

For use in non-Cargo builds like Bazel or Buck, CXX provides an alternate way of
invoking the C++ code generator as a standalone command line tool. The tool is
packaged as the `cxxbridge-cmd` crate on crates.io or can be built from the
*cmd* directory of this repo.

```bash
$ cargo install cxxbridge-cmd

$ cxxbridge src/main.rs --header > path/to/mybridge.h
$ cxxbridge src/main.rs > path/to/mybridge.cc
```

<br>

## Safety

Be aware that the design of this library is intentionally restrictive and
opinionated! It isn't a goal to be powerful enough to handle arbitrary
signatures in either language. Instead this project is about carving out a
reasonably expressive set of functionality about which we can make useful safety
guarantees today and maybe extend over time. You may find that it takes some
practice to use CXX bridge effectively as it won't work in all the ways that you
are used to.

Some of the considerations that go into ensuring safety are:

- By design, our paired code generators work together to control both sides of
  the FFI boundary. Ordinarily in Rust writing your own `extern "C"` blocks is
  unsafe because the Rust compiler has no way to know whether the signatures
  you've written actually match the signatures implemented in the other
  language. With CXX we achieve that visibility and know what's on the other
  side.

- Our static analysis detects and prevents passing types by value that shouldn't
  be passed by value from C++ to Rust, for example because they may contain
  internal pointers that would be screwed up by Rust's move behavior.

- To many people's surprise, it is possible to have a struct in Rust and a
  struct in C++ with exactly the same layout / fields / alignment / everything,
  and still not the same ABI when passed by value. This is a longstanding
  bindgen bug that leads to segfaults in absolutely correct-looking code
  ([rust-lang/rust-bindgen#778]). CXX knows about this and can insert the
  necessary zero-cost workaround transparently where needed, so go ahead and
  pass your structs by value without worries. This is made possible by owning
  both sides of the boundary rather than just one.

- Template instantiations: for example in order to expose a UniquePtr\<T\> type
  in Rust backed by a real C++ unique\_ptr, we have a way of using a Rust trait
  to connect the behavior back to the template instantiations performed by the
  other language.

[rust-lang/rust-bindgen#778]: https://github.com/rust-lang/rust-bindgen/issues/778

<br>

## Builtin types

In addition to all the primitive types (i32 ⟷ int32_t), the following common
types may be used in the fields of shared structs and the arguments and returns
of functions.

<table>
<tr><th>name in Rust</th><th>name in C++</th><th>restrictions</th></tr>
<tr><td>String</td><td>rust::String</td><td></td></tr>
<tr><td>&amp;str</td><td>rust::Str</td><td></td></tr>
<tr><td><a href="https://docs.rs/cxx/0.1/cxx/struct.CxxString.html">CxxString</a></td><td>std::string</td><td><sup><i>cannot be passed by value</i></sup></td></tr>
<tr><td>Box&lt;T&gt;</td><td>rust::Box&lt;T&gt;</td><td><sup><i>cannot hold opaque C++ type</i></sup></td></tr>
<tr><td><a href="https://docs.rs/cxx/0.1/cxx/struct.UniquePtr.html">UniquePtr&lt;T&gt;</a></td><td>std::unique_ptr&lt;T&gt;</td><td><sup><i>cannot hold opaque Rust type</i></sup></td></tr>
<tr><td></td><td></td><td></td></tr>
</table>

The C++ API of the `cxxbridge` namespace is defined by the *include/cxxbridge.h*
file in this repo. You will need to include this header in your C++ code when
working with those types.

The following types are intended to be supported "soon" but are just not
implemented yet. I don't expect any of these to be hard to make work but it's a
matter of designing a nice API for each in its non-native language.

<table>
<tr><th>name in Rust</th><th>name in C++</th></tr>
<tr><td>&amp;[T]</td><td><sup><i>tbd</i></sup></td></tr>
<tr><td>Vec&lt;T&gt;</td><td><sup><i>tbd</i></sup></td></tr>
<tr><td>BTreeMap&lt;K, V&gt;</td><td><sup><i>tbd</i></sup></td></tr>
<tr><td>HashMap&lt;K, V&gt;</td><td><sup><i>tbd</i></sup></td></tr>
<tr><td><sup><i>tbd</i></sup></td><td>std::vector&lt;T&gt;</td></tr>
<tr><td><sup><i>tbd</i></sup></td><td>std::map&lt;K, V&gt;</td></tr>
<tr><td><sup><i>tbd</i></sup></td><td>std::unordered_map&lt;K, V&gt;</td></tr>
</table>

<br>

## Remaining work

This is still early days for CXX; I am releasing it as a minimum viable product
to collect feedback on the direction and invite collaborators. Here are some of
the facets that I still intend for this project to tackle:

- [ ] Support associated methods: `extern "Rust" { fn f(self: &Struct); }`
- [ ] Support C++ member functions
- [ ] Support passing function pointers across the FFI
- [ ] Support translating between Result ⟷ exceptions
- [ ] Support structs with type parameters
- [ ] Support async functions

On the build side, I don't have much experience with the `cc` crate so I expect
there may be someone who can suggest ways to make that aspect of this crate
friendlier or more robust. Please report issues if you run into trouble building
or linking any of this stuff.

Finally, I know more about Rust library design than C++ library design so I
would appreciate help making the C++ APIs in this project more idiomatic where
anyone has suggestions.

<br>

#### License

<sup>
Licensed under either of <a href="LICENSE-APACHE">Apache License, Version
2.0</a> or <a href="LICENSE-MIT">MIT license</a> at your option.
</sup>

<br>

<sub>
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this project by you, as defined in the Apache-2.0 license,
shall be dual licensed as above, without any additional terms or conditions.
</sub>

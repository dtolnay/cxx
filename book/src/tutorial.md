# Tutorial

We're going to build a Rust project which uses a C++ parser for California street addresses. That parser will need to take a C++ string, and return a structure that has the house number and the street name.

We'll passing a C++ string in one direction, and a struct in the other.

We'll also validate the house number by calling a separate Rust function.

## Creating the project

Create a blank Cargo project: `mkdir cxx-demo`; `cd cxx-demo`; `cargo init`.

Edit the `Cargo.toml` to add a depdendency on `cxx`:

```toml
[dependencies]
cxx = "0.4.7"
```

(we'll revisit this `Cargo.toml` later when we want to build the C++ code.)

## Creating the FFI section

Unlike [tools such as `bindgen` or `cbindgen`](context.md), `cxx` requires an extra declaration or definition for all functions and types which you wish to pass across the FFI boundary. That's necessary so that `cxx` can generate the safe interop layers on _both_ sides of the boundary, unlike those other tools.

Let's start by stating those interfaces, and then we'll fill out both C++ and Rust sides to match these interfaces.

In `src/main.rs`, add a `#[cxx::bridge]` mod like this at the top of the file:

```rust
#[cxx::bridge]
mod ffi {

}
```

We will fill out this section with _everything_ that needs to be known to both sides of the FFI boundary.

## Filling in types

We want our address parsing function to pass its results back from C++ to Rust in a struct, so we need to define this struct here.

```rust
#[cxx::bridge]
mod ffi {
    struct Address {
        house_number: u64, // California house numbers are ridiculous
        street: UniquePtr<CxxString>,
    }
}
```

What's with the `UniquePtr<CxxString>` bit? These are Rust names for standard C++ types - specifically, `std::unique_ptr<T>` and `std::string`. See [C++ types](cpp-types.md) for more information, or see [this list of types](https://docs.rs/cxx/0.4.7/cxx/#builtin-types) for a full reference.

Meanwhile though, note that the C++ string is not being held in Rust by value. This is important. A C++ string is a hairy, scary type which might contain pointers, possibly self-referential pointers at that. It can't be safely included in a Rust structure, because Rust is free to move memory at any time.

For that reason, Rust code can only safely hold a `std::string` by pointer - and that's just what we're doing.

Conversely, if our struct contained only 'trivial' C++ types (e.g. integers, or other structs which are also trivial) we can own them and pass them around in Rust quite happily. See [C++ types](cpp-types.md) for more information.

But, if in doubt, own scary C++ types by `UniquePtr` within Rust.

## What does this generate?

To understand what's going on here, you can expand the macro to see what it generates. This is not necessary in normal `cxx` usage, but for the purposes of this tutorial it may help understand what `cxx` is doing.

```
cargo install cargo-expand
cargo expand --manifest-path cxx-demo/Cargo.toml
```

You'll see it has generated a `#[repr(C)]` version of this struct.

```rust
mod ffi {
    #[derive()]
    #[repr(C)]
    pub struct Address {
        pub house_number: u64,
        pub street: ::cxx::UniquePtr<::cxx::CxxString>,
    }
}
```

Later, we'll see how the very same FFI bindings will generate equivalent C++ headers with precisely matching definitions and memory layout.

## Adding our calls from Rust into C++

Expand your `mod ffi` so it looks like this:

```rust
#[cxx::bridge]
mod ffi {
    struct Address {
        house_number: u64,
        street: UniquePtr<CxxString>,
    }

    extern "C++" {
        include!("include/demo.h");
        fn parse_address(input: &str) -> Address;
    }
}
```

`include/demo.h` is the file in which `parse_address` will be declared in C++. We're going to implement that C++ shortly.

Again, you can use `cargo expand` to see what's generated. You'll see that `cxx` is generating lots of _deeply unpleasant_ code involving `unsafe`, `MaybeUninit`, `#[link_name]` and all kinds of nastiness.

It's normal in Rust for low-level types and libraries to use `unsafe` internally, but by virtue of human checks and cunning, present a safe interface to their customers. That's exactly what's happening here in `cxx` as well. Because `cxx` generates both the C++ and Rust sides of this interop boundary, it can guarantee that the interop is safe. As a user of `cxx` you can therefore use this interface without worrying about all these details. See the book chapter on [safety](safety.md) for more details.

We'll never look at the `cargo expand` output again.

## Using our C++ call from Rust

Let's change our `fn main()` to look like this:

```rust
fn main() {
    let input_address = &std::env::args().collect::<Vec<String>>()[1];
    let parsed = ffi::parse_address(input_address);
    println!("Street is: {}. Number is {}.", parsed.street, parsed.house_number);
}
```

Note:
* You can use the FFI struct in a very natural way
* The `UniquePtr<CxxString>` doesn't require any special hoop-jumping.
* There's no need to declare `unsafe` because `cxx` can prove you're not doing anything unsafe in your code. (All its unsafety is internal. Again, see [safety](safety.md) for more details.)

If you build this, it won't link. Specifically, it will complain with something like:

```
  = note: Undefined symbols for architecture x86_64:
            "_cxxbridge04$parse_address", referenced from:
                cxx_demo::ffi::parse_address::h1410eab40f95ba57 in cxx_demo.1zsecbvpgtf4ntoh.rcgu.o
```

It's absolutely right! We haven't implemented the C++ side yet.

## Building the C++ side

As a user of `cxx` you probably have a pre-existing sophisticated C++ build system. Check out the chapter on [building](building.md) for how you can integrate `cxx` into that system.
You probably _won't_ be doing the following, but let's do it for the tutorial.

In your `Cargo.toml` add a build dependency:

```toml
[build-dependencies]
cxx-build = "0.4.7"
```

and add a `build.rs` file alongside the `Cargo.toml`:

```rust
fn main() {
    cxx_build::bridge("src/main.rs")
        .file("src/demo.cc")
        .flag_if_supported("-std=c++14")
        .compile("cxx-demo");

    println!("cargo:rerun-if-changed=src/main.rs");
    println!("cargo:rerun-if-changed=src/demo.cc");
    println!("cargo:rerun-if-changed=include/demo.h");
}
```

Create a blank `src/demo.cc` and `include/demo.h`. Run `cargo build`. You should get complaints that `parse_address` is an undeclared identifier.

## On the generated C++ code

`cxx` will auto-generate some C++ code, which you can look at within `target/cxxbridge/cxx-demo/src/main.rs.h`. Amongst other things, you'll see a definition of the `Address` struct:

```c++
struct Address final {
  uint64_t house_number;
  ::std::unique_ptr<::std::string> street;
};
```

You shouldn't normally need to look at this generated C++ code, but the key thing to remember is that these definitions precisely match those generated by the expansion of the Rust macro.

Obviously, it doesn't contain an implementation of the `parse_address` function, so that's what we'll do next.

## Implementing the C++ side

Put this into `include/demo.h`:

```c++
#pragma once
#include "rust/cxx.h"
#include "cxx-demo/src/main.rs.h"

Address parse_address(::rust::Str input);                                             
```

and put this into `src/demo.cc`:

```c++
#include "cxx-demo/include/demo.h"

Address parse_address(::rust::Str input) {
    auto to_parse = std::string(input);
    Address results;
    // ...
    return results;
}
```

Note that your C++ code is including `main.rs.h` which we inspected a moment ago. That's the key part: this contains definitions of those structures which you defined in your `#[cxx::bridge]` section back in `main.rs`. In this way, `cxx` can be sure that both Rust and C++ sides are working on the same data types.

Finally, `cargo run -- "99999 Frogmella"`.

## A little more on unsafety...

The project _should_ build and run. As part of the Cargo build procedure, the instructions in `build.rs` are invoking your C++ compiler to build your C++ code too.

You'll probably see output like:

```
Street is: nullptr. Number is 140403911187472.
```

So: Rust cannot make C++ code fully safe. Here, the C++ code is making a mistake - accessing uninitialized data - and Rust is relaying the results of that mistake. _cxx cannot magically make C++ safe_, and mistakes in C++ can have consequences in Rust code. See [safety](safety.md) for more.

## Completing the program

Let's fix that bug in the C++:

```c++
#include "cxx-demo/include/demo.h"
#include <sstream>

Address parse_address(::rust::Str input) {
    auto to_parse = std::string(input);
    Address results;
    std::stringstream ss(to_parse);
    std::string street_address;
    ss >> results.house_number >> street_address;
    results.street = std::make_unique<std::string>(street_address);
    return results;
}
```

(Yes, obviously the street address can only be a single word long in this example. I don't care. If you've got this far and are seriously considering writing a parser in C++ rather than Rust, you need to reconsider your life choices.)

```
$ cargo run -- "99999 Frogmella"
Street is: Frogmella. Number is 99999.
```

Ta-da!

## Calls from C++ to Rust

Let's enhance our program to call from C++ back to Rust, to validate the house number.

Modify the `#[cxx::bridge]` to add an `extern "Rust"` section like this:

```rust
#[cxx::bridge]
mod ffi {
    struct Address {
        house_number: u64,
        street: UniquePtr<CxxString>,
    }

    extern "C++" {
        include!("cxx-demo/include/demo.h");
        fn parse_address(input: &str) -> Address;
    }

    extern "Rust" {
        fn validate_house_number(number: u64);
    }
}
```

Implement that function elsewhere in `main.rs`.

```rust
fn validate_house_number(number: u64) {
    if number < 10000 {
        panic!("Unrealistically small California house number.");
    }
}
```

(see [the chapter on error handling for better strategies!](error-handling.md)).

From C++, we can simply call this function:

```c++
validate_house_number(results.house_number);
```

You should now see that

```
cargo run -- "99999 Frogmella"
```

works, whereas

```
cargo run -- "99 Frogmella"
```

dies horribly.

## Next steps

Thanks for reading this tutorial! To learn more, a good next stopping point is probaly the chapter on [C++ types and calls](cpp-types.md). But consider also stopping off at the pages about [safety](safety.md), [error handling](error-handling.md) or [Rust calls and types](rust-types.md).

# About cxx

`cxx` provides a safe mechanism for calling C++ code from Rust and Rust code from C++, not subject to the many ways that things can go wrong when using bindgen or cbindgen to generate unsafe C-style bindings.

It achieves this safety by generating code on _both_ the C++ and Rust side, ensuring that you can pass types safely from one side to the other. You can manipulate [C++ types](cpp-types.md) from Rust, and [Rust types](rust-types.md) from C++.

<img src="overview.svg">

# Minimal example

```rust
#[cxx::bridge]
mod ffi {
    extern "C" {
        include!("demo/include/demo.h");

        type ThingC;

        // Functions implemented in C++.
        fn make_demo(appname: &str) -> UniquePtr<ThingC>;
        fn do_something(obj: &ThingC);
    }
}

fn my_normal_rust_code() {
    let cxx_thing = ffi::make_demo("Slartibartfast");
    ffi::do_something(cxx_thing.as_ref().unwrap());
    // ...
}
```

# More examples

For more examples see:

* [README.md](https://docs.rs/cxx/0.4.7/cxx/#example)
* The demo directory:
    - [demo/src/main.rs](https://github.com/dtolnay/cxx/blob/master/demo/src/main.rs)
    - [demo/build.rs](https://github.com/dtolnay/cxx/blob/master/demo/build.rs)
    - [demo/include/demo.h](https://github.com/dtolnay/cxx/blob/master/demo/include/demo.h)
    - [demo/src/demo.cc](https://github.com/dtolnay/cxx/blob/master/demo/src/demo.cc)
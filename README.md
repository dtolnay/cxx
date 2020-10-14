# CXX with CMake build system

This is an example repo to setup [cxx](https://github.com/dtolnay/cxx) with cmake build system.

The official [demo](https://github.com/dtolnay/cxx/tree/master/demo) used `cargo` to orchestrate the two build systems and place the `main` function inside rust project.

In a lot other applications, however, we want to embed rust into a large cpp project where we don't have a chance to choose build systems.
This template repo shows how to use cmake with cxx.


The cmake files do the following things:
1. Call `cargo build [--release]` to build a shared libraray
2. Call `cxxbridge src/lib.rs > ...` to generate the header file (as specified [here](https://github.com/dtolnay/cxx#non-cargo-setup))
3. Copy the shared lib as well as the headers to CMAKE_BINARY_DIR
4. Link and include the libraray

The cmake files are largely insipred by [Using unsafe for Fun and Profit](https://github.com/Michael-F-Bryan/rust-ffi-guide).

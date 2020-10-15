# CXX with CMake build system

This is an example repo to setup [cxx](https://github.com/dtolnay/cxx) with the cmake build system.

The official [demo](https://github.com/dtolnay/cxx/tree/master/demo) used `cargo` to orchestrate the two build systems and place the `main` function inside the rust project.

In a lot of other applications, however, we want to embed rust into a large cpp project where we don't have a chance to choose build systems.
This template repo shows how to use cmake with cxx.


The cmake files do the following things:
1. Call `cargo build [--release]` to build a shared library
2. Call `cxxbridge src/lib.rs > ...` to generate the source/header files (as specified [here](https://github.com/dtolnay/cxx#non-cargo-setup))
3. Create a shared lib from the cxx generated source and link to the rust .so file
3. Copy the shared libs as well as the headers to CMAKE_BINARY_DIR
4. Link and include the libraray to the corresponding targets

The cmake files are largely inspired by [Using unsafe for Fun and Profit](https://github.com/Michael-F-Bryan/rust-ffi-guide).

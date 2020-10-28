# CXX with CMake build system

This is an example repo to setup [cxx](https://github.com/dtolnay/cxx) with the cmake build system.

The official [demo](https://github.com/dtolnay/cxx/tree/master/demo) used `cargo` to orchestrate the two build systems and place the `main` function inside the rust project.

In a lot of other applications, however, we want to embed rust into a large cpp project where we don't have a chance to choose build systems.
This template repo shows how to use cmake with cxx.


The cmake files do the following things:
1. Call `cargo build [--release]` to build a static library
2. Call `cxxbridge src/lib.rs > ...` to generate the source/header files (as specified [here](https://github.com/dtolnay/cxx#non-cargo-setup))
3. Create a library from the cxx generated source and link to the rust static library
4. Link and include the libraray to the corresponding targets


## Cross-language LTO (linking time optimization)
### Why?
Calling rust function from c++ (and vice versa) is not zero-overhead because the LLVM optimizer by default will not optimize across shared libraries, let alone across different languages.

[LTO](https://llvm.org/docs/LinkTimeOptimization.html) (linking time optimization) allows the optimizers to perform optimization during linking time (instead of compile time), thus enables cross library optimization.

### How?
```bash
cmake -DENABLE_LTO=ON -DCMAKE_BUILD_TYPE=Release ..
make -j
./main
```

The `-DENABLE_LTO=ON` will compile and link both libraries with proper parameters.
Note that cross language LTO between rust and c/c++ is only possible with clang toolchain, 
meaning that you need to have a very recent clang/lld installed.

## Example output

### With LTO
```
Points {
    x: [
        1,
        2,
        3,
    ],
    y: [
        4,
        5,
        6,
    ],
}
"cpp expert!"
Calling rust function, time elapsed: 100 ns.
Calling c++ function, time elapsed: 100 ns.
```

#### Without LTO
```
Points {
    x: [
        1,
        2,
        3,
    ],
    y: [
        4,
        5,
        6,
    ],
}
"cpp expert!"
Calling rust function, time elapsed: 1176600 ns.
Calling c++ function, time elapsed: 100 ns.
```


## Credits
The cmake files are largely inspired by [Using unsafe for Fun and Profit](https://github.com/Michael-F-Bryan/rust-ffi-guide).

I learned a lot about cross language LTO from this post: [Closing the gap: cross-language LTO between Rust and C/C++](https://blog.llvm.org/2019/09/closing-gap-cross-language-lto-between.html)

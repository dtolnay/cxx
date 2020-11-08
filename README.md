# Rust CXX demo with CBindgen, and CMake on Windows and Linux

This is an example project to experment and show how to use Rust and
C++ together in a single shared library, using C++ strengths and Rust
for it's strengths too. 

- C++ can call Rust functions, with native data types (such as
  `std::unique_ptr`).
- Rust can call C++ functions, with native data types (such as
  `std::Box`)
- C++ can call C-compatible Rust functions.

# Compile and Test (GNU Linux)

The build environment assumes to be running on Linux with GCC 6.3.1.

``` shell
$ cd /path/to/project/root/

$ bash build_linux.bash

# Run tests
$ ./install/bin/mmscenegraph_tests
2 + 2 = 4
SCENEGRAPH: Add SceneGraph 42
SCENEGRAPH: Remove SceneGraph 0x96b5b0
my awesome demo
done with ThingC
```

Note: If you are using CentOS 7.8 and require the Red Hat Developer
Toolset 6 (GCC 6.3.1) for compatiblity with the VFX Reference Platform CY2018,
then you cause use the following commands prior to running `build_linux.bash`:
```
$ export CC=/opt/rh/devtoolset-6/root/usr/bin/gcc
$ export CXX=/opt/rh/devtoolset-6/root/usr/bin/g++
$ export _GLIBCXX_USE_CXX11_ABI=0  # Use the old std::string and std::list ABI.

# Enable Red Hat Developer Toolset 6
$ scl enable devtoolset-6 bash
```

# Compile and Test (Microsoft Windows)

This has been tested on Windows 10, with Visual Studio 2015.

Make sure the following commands are run in the Visual Studio 2015 enabled Command Prompt.
``` shell
> cd C:\path\to\project\root\

> build_windows64.bat

> install\bin\mmscenegraph_tests.exe
2 + 2 = 4
SCENEGRAPH: Add SceneGraph 42
SCENEGRAPH: Remove SceneGraph 0x1ffc8819410
my awesome demo
done with ThingC
```

# Dependencies

You will need the Rust compiler and a C/C++ compiler.  Below are the
tested dependancy versions, for both Linux and Windows (MacOSX has not
been tested).

This demo targets the C++11 standard.

- Linux
  - GCC 6.3.1
  - CMake 2.8.12+
  - Rust 1.47
    - cbindgen
    - cxx 0.5.5
    - libc
  - cxxbridge-cmd 0.5.5 (installed via Rust's Cargo package manager)
- Windows 10
  - MSVC - Visual Studio 2015
  - CMake 2.8.12+
  - Rust 1.47
    - cbindgen
    - cxx 0.5.5
    - libc
  - cxxbridge-cmd 0.5.5 (installed via Rust's Cargo package manager)

# Project Anatomy

This example project contains many different files needing to be
compiled by the Rust compilier and a C++11 compliant compilier.

## Rust code

The Rust code is compiled by the `cargo` command, and starts in the
file `src/lib.rs`. Any files needing to be compiled, such as
`src/ffi.rs` and `src/cxxbridge.rs` must be loaded using the `pub mod`
keywords.

Simply add more modules to add Rust functions and structs to the Rust
side of the project.

## C++ Files

The C++ code is compiled and linked with the Rust code, after the Rust
compiler has produced a static library, the C++ compiler then links
the Rust and C++ libraries together into a single Shared library (.so
on Linux and .dll/.lib on Windows).

Files:
```
src/lib.cpp
include/mmscenegraph/_cpp.h
```

## Library Header

The `include/mmscenegraph.h` file defines the functions, data
structures and data types from Rust and C++ into a C++ compatible
header file.

`include/mmscenegraph.h` is publically facing and has a C++ namespace
`mmscenegraph`.

The contents of `include/mmscenegraph.h` combines auto-generated
headers, see the next section for details.

## Auto-Generated C++ code

Rust source files are parsed by the commands `cbindgen` and
`cxxbridge`. The commands are used to generate C++ source and header
files which can then be used in a C++ compiler.

The details of the `cbindgen` command is controlled by the
`cbindgen.toml` configuration file, and Rust code in defined in
`src/ffi.rs`.

The `cxxbridge` command parses the `src/cxxbridge.rs` file and outputs
`src/_cxxbridge.cpp` and `include/_cxxbridge.h` files. `cxxbridge` Can
also generate a C++ shim header `include/cxx.h` to aid in the
transition between C++ and Rust functions.

Files:
```
src/ffi.rs
src/cxxbridge.rs
src/_cxxbridge.cpp
include/cxx.h
include/_cbindgen.h
include/_cxxbridge.h
cbindgen.toml
```

## Build files

This project is built by both `cargo` (for Rust code) and `cmake` (for
C++ code). To combine the two programs in an easy-to-understand way,
the `build_linux.bash` GNU Bash and `build_windows64.bat` Windows
Batch file are provided.

CMake is used to define the C++ build and must be called after all
files have been generated and Rust code has been compiled. CMake
2.8.12 is the minimum supported version, due to backwards
compatibility. The configuation file is `CMakeLists.txt`.

`Cargo.toml` defines the Rust compiler's dependencies and build
process.

Files:
```
CMakeLists.txt
Cargo.toml
build_linux.bash
build_windows64.bat
```

## Tests

To ensure everything is working the `tests` sub-directory contains
some example client code using both the `cbindgen` and `cxx`
functions.

Files:
```
tests/test_a.cpp (cbindgen tests)
tests/test_b.cpp (cxx tests)
```

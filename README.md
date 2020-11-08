# Rust CXX demo with CBindgen, and CMake on Windows and Linux

This is an example project to experment and show how to use Rust and
C++ together in a single shared library, using C++ strengths and Rust
for it's strengths too. 

# Compile and Test (GNU Linux)

The build environment assumes to be running on CentOS Linux release
7.8.2003 with GCC 6.3.1 as the C++ compiler (using Red Hat Developer
Toolset 6), and aims for compatiblity with the VFX Reference Platform
CY2018. https://vfxplatform.com/

``` shell
$ cd /path/to/project/root/

# Enable Red Hat Developer Toolset 6
$ scl enable devtoolset-6 bash

$ bash build_linux.bash

# Run tests
$ ./install/bin/mmscenegraph_tests
2 + 2 = 4
SCENEGRAPH: Add SceneGraph 42
SCENEGRAPH: Remove SceneGraph 0x96b5b0
my awesome demo
done with ThingC
```

# Compile and Test (Microsoft Windows)

This has been tested on Windows 10, with Visual Studio 2015.

Make sure the following commands are run in the Visual Studio 2015 enabled Command Prompt.
``` shell
> CHDIR C:\path\to\project\root\

> build_windows64.bat

> install\bin\mmscenegraph_tests.exe
```

# Dependencies

You will need the Rust compiler and a C/C++ compiler.  Below are the
tested dependancy versions, for both Linux and Windows (MacOSX has not
been tested).

- Linux - CentOS 7
  - GCC 6.3.1
  - CMake 2.8.12+
  - Rust 1.47+
    - cbindgen
    - cxx
    - libc
- Windows 10
  - MSVC - Visual Studio 2015
  - CMake 2.8.12+
  - Rust 1.47+
    - cbindgen
    - cxx
    - libc


This directory contains CXX's C++ code generator. This code generator has two
public frontends, one a command-line application (binary) in the *cmd* directory
and the other a library intended to be used from a build.rs in the *build*
directory.

There's also a 'lib' frontend which is intended to allow higher level code
generators to embed cxx. This is not yet recommended for general use.

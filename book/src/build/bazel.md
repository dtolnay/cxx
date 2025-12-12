{{#title Bazel, Buck2 — Rust ♡ C++}}
## Bazel, Buck2, potentially other similar environments

Starlark-based build systems with the ability to compile a code generator and
invoke it as a `genrule` will run CXX's C++ code generator via its `cxxbridge`
command line interface.

The tool is packaged as the `cxxbridge-cmd` crate on crates.io or can be built
from the *gen/cmd/* directory of the CXX GitHub repo.

```console
$  cargo install cxxbridge-cmd

$  cxxbridge src/bridge.rs --header > path/to/bridge.rs.h
$  cxxbridge src/bridge.rs > path/to/bridge.rs.cc
```

<div class="warning">

**Important:** The version number of `cxxbridge-cmd` used for the C++ side of
the binding must always be identical to the version number of `cxx` used for the
Rust side. You must use some form of lockfile or version pinning to ensure that
this is the case.

</div>

The CXX repo maintains working [Bazel] `BUILD.bazel` and [Buck2] `BUCK` targets
for the complete blobstore tutorial (chapter 3) for your reference, tested in
CI. These aren't meant to be directly what you use in your codebase, but serve
as an illustration of one possible working pattern.

[Bazel]: https://bazel.build
[Buck2]: https://buck2.build

```python
# tools/bazel/rust_cxx_bridge.bzl

{{#include ../../../tools/bazel/rust_cxx_bridge.bzl}}
```

```python
# demo/BUILD.bazel

{{#include ../../../demo/BUILD.bazel}}
```

```python
# BUILD.bazel

{{#include ../../../BUILD.bazel}}
```

```python
# MODULE.bazel

{{#include ../../../MODULE.bazel}}
```

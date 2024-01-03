workspace(name = "cxx.rs")

load("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")

http_archive(
    name = "rules_rust",
    integrity = "sha256-p2HVTknbBvhjRo5rukoTJSsb1Jno9wbaZeJ5s7y8XFI=",
    urls = ["https://github.com/bazelbuild/rules_rust/releases/download/0.36.2/rules_rust-v0.36.2.tar.gz"],
)

load("@rules_rust//rust:repositories.bzl", "rules_rust_dependencies", "rust_register_toolchains")

rules_rust_dependencies()

rust_register_toolchains(
    versions = ["1.75.0"],
)

load("@rules_rust//crate_universe:repositories.bzl", "crate_universe_dependencies")

crate_universe_dependencies()

load("//third-party/bazel:defs.bzl", "crate_repositories")

crate_repositories()

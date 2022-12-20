workspace(name = "cxx.rs")

load("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")

http_archive(
    name = "rules_rust",
    sha256 = "05e15e536cc1e5fd7b395d044fc2dabf73d2b27622fbc10504b7e48219bb09bc",
    urls = ["https://github.com/bazelbuild/rules_rust/releases/download/0.8.1/rules_rust-v0.8.1.tar.gz"],
)

load("@rules_rust//rust:repositories.bzl", "rules_rust_dependencies", "rust_register_toolchains")

RUST_VERSION = "1.66.0"

rules_rust_dependencies()

rust_register_toolchains(
    version = RUST_VERSION,
)

load("//tools/bazel:vendor.bzl", "vendor")

vendor(
    name = "third-party",
    cargo_version = RUST_VERSION,
    lockfile = "//third-party:Cargo.lock",
)

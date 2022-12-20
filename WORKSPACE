workspace(name = "cxx.rs")

load("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")

http_archive(
    name = "rules_rust",
    sha256 = "dd79bd4e2e2adabae738c5e93c36d351cf18071ff2acf6590190acf4138984f6",
    urls = ["https://github.com/bazelbuild/rules_rust/releases/download/0.14.0/rules_rust-v0.14.0.tar.gz"],
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

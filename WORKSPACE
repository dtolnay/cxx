workspace(name = "cxx.rs")

load("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")

http_archive(
    name = "rules_rust",
    sha256 = "324c2a86a8708d30475f324846b35965c432b63a35567ed2b5051b86791ce345",
    urls = ["https://github.com/bazelbuild/rules_rust/releases/download/0.13.0/rules_rust-v0.13.0.tar.gz"],
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

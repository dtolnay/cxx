workspace(name = "cxx.rs")

load("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")

http_archive(
    name = "rules_rust",
    sha256 = "29954bced3e0d1a57ff8db816f5cd8a5856179fc657455729f1eb53b39611419",
    strip_prefix = "rules_rust-6e1cbbfcd0d140baacc8ff1080f885d2a45296a9",
    urls = [
        # PR https://github.com/bazelbuild/rules_rust/pull/1254
        # on top of the main branch as of 2022-04-10
        "https://github.com/bazelbuild/rules_rust/archive/6e1cbbfcd0d140baacc8ff1080f885d2a45296a9.tar.gz",
    ],
)

load("@rules_rust//rust:repositories.bzl", "rust_repositories")

RUST_VERSION = "1.60.0"

rust_repositories(
    edition = "required",
    version = RUST_VERSION,
)

load("//tools/bazel:vendor.bzl", "vendor")

vendor(
    name = "third-party",
    lockfile = "//third-party:Cargo.lock",
    cargo_version = RUST_VERSION,
)

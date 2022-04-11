workspace(name = "cxx.rs")

load("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")

http_archive(
    name = "rules_rust",
    sha256 = "3cf493f845837b9c0c44311992a8e387b508a267cb8f261ef97b94c915f292cc",
    strip_prefix = "rules_rust-55790492aca01b389d208cd1335b9d8c05e28329",
    urls = [
        # Main branch as of 2022-04-10
        "https://github.com/bazelbuild/rules_rust/archive/55790492aca01b389d208cd1335b9d8c05e28329.tar.gz",
    ],
)

load("@rules_rust//rust:repositories.bzl", "rust_repositories")

RUST_VERSION = "1.60.0"

rust_repositories(version = RUST_VERSION)

load("//tools/bazel:vendor.bzl", "vendor")

vendor(
    name = "third-party",
    lockfile = "//third-party:Cargo.lock",
    cargo_version = RUST_VERSION,
)

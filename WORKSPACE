load("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")

http_archive(
    name = "io_bazel_rules_rust",
    sha256 = "b83154a58f95618e06845b774b079000e0c39830e185db4c7bf46e79896cb3a1",
    strip_prefix = "rules_rust-0deef6dd8180cd3bc610878558bb26921b4e8de1",
    # Master branch as of 2020-03-07
    url = "https://github.com/bazelbuild/rules_rust/archive/0deef6dd8180cd3bc610878558bb26921b4e8de1.tar.gz",
)

http_archive(
    name = "bazel_skylib",
    sha256 = "97e70364e9249702246c0e9444bccdc4b847bed1eb03c5a3ece4f83dfe6abc44",
    urls = [
        "https://mirror.bazel.build/github.com/bazelbuild/bazel-skylib/releases/download/1.0.2/bazel-skylib-1.0.2.tar.gz",
        "https://github.com/bazelbuild/bazel-skylib/releases/download/1.0.2/bazel-skylib-1.0.2.tar.gz",
    ],
)

load("@io_bazel_rules_rust//:workspace.bzl", "bazel_version")

bazel_version(name = "bazel_version")

load("@io_bazel_rules_rust//rust:repositories.bzl", "rust_repository_set")

rust_repository_set(
    name = "rust_1_43_linux",
    exec_triple = "x86_64-unknown-linux-gnu",
    version = "1.43.0",
)

rust_repository_set(
    name = "rust_1_43_darwin",
    exec_triple = "x86_64-apple-darwin",
    version = "1.43.0",
)

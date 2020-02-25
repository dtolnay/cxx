load("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")

local_repository(
    name = "io_bazel_rules_rust",
    path = "../bazel-rules-rust",
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
    name = "rust_1_42_beta_linux",
    exec_triple = "x86_64-unknown-linux-gnu",
    extra_target_triples = [],
    iso_date = "2020-02-08",
    version = "beta",
)

rust_repository_set(
    name = "rust_1_42_beta_darwin",
    exec_triple = "x86_64-apple-darwin",
    extra_target_triples = [],
    iso_date = "2020-02-08",
    version = "beta",
)

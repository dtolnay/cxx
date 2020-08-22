load("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")

http_archive(
    name = "io_bazel_rules_rust",
    sha256 = "5ed804fcd10a506a5b8e9e59bc6b3b7f43bc30c87ce4670e6f78df43604894fd",
    strip_prefix = "rules_rust-fdf9655ba95616e0314b4e0ebab40bb0c5fe005c",
    # Master branch as of 2020-07-30
    url = "https://github.com/bazelbuild/rules_rust/archive/fdf9655ba95616e0314b4e0ebab40bb0c5fe005c.tar.gz",
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
    name = "rust_1_45_linux",
    exec_triple = "x86_64-unknown-linux-gnu",
    version = "1.45.0",
)

rust_repository_set(
    name = "rust_1_45_darwin",
    exec_triple = "x86_64-apple-darwin",
    version = "1.45.0",
)

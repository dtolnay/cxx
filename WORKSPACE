workspace(name = "cxx.rs")

load("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")
load("//tools/bazel:vendor.bzl", "vendor")

http_archive(
    name = "rules_rust",
    sha256 = "697a6f4f2adbd1b00f792346d6eca4cd45f691be63069c7d7ebf4fcf82a377a8",
    strip_prefix = "rules_rust-8bad4c5e4e53d9f6f8d4d5228e26a44d92f37ab2",
    urls = [
        # Master branch as of 2021-04-11
        "https://github.com/bazelbuild/rules_rust/archive/8bad4c5e4e53d9f6f8d4d5228e26a44d92f37ab2.tar.gz",
    ],
)

load("@rules_rust//rust:repositories.bzl", "rust_repositories")

rust_repositories(
    edition = "2018",
    version = "1.51.0",
)

vendor(
    name = "third-party",
    lockfile = "//third-party:Cargo.lock",
)

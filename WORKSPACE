workspace(name = "cxx.rs")

load("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")

http_archive(
    name = "rules_rust",
    sha256 = "d39af65fc3b88c204e101d4ddc7dd7543be4e2509fa21c35a5d228f54821a2a9",
    strip_prefix = "rules_rust-f7cb22efa64a6a07813e30e9f9d70d1fd18e463e",
    urls = [
        # Main branch as of 2022-04-25
        "https://github.com/bazelbuild/rules_rust/archive/f7cb22efa64a6a07813e30e9f9d70d1fd18e463e.tar.gz",
    ],
)

load("@rules_rust//rust:repositories.bzl", "rules_rust_dependencies", "rust_register_toolchains")

RUST_VERSION = "1.60.0"

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

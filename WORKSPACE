workspace(name = "cxx.rs")

load("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")

http_archive(
    name = "rules_rust",
    sha256 = "48e715be2368d79bc174efdb12f34acfc89abd7ebfcbffbc02568fcb9ad91536",
    urls = ["https://github.com/bazelbuild/rules_rust/releases/download/0.24.0/rules_rust-v0.24.0.tar.gz"],
)

load("@rules_rust//rust:repositories.bzl", "rules_rust_dependencies", "rust_register_toolchains")

rules_rust_dependencies()

rust_register_toolchains(
    versions = ["1.69.0"],
)

load("@rules_rust//crate_universe:repositories.bzl", "crate_universe_dependencies")

crate_universe_dependencies()

load("//third-party/bazel:defs.bzl", "crate_repositories")

crate_repositories()

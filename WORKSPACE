workspace(name = "cxx.rs")

load("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")

http_archive(
    name = "rules_rust",
    sha256 = "0f18dd752b87d2203c140b3e356364b08a91eb6aa9b2d689ea69eb7cc2530f4d",
    urls = ["https://github.com/bazelbuild/rules_rust/releases/download/0.33.0/rules_rust-v0.33.0.tar.gz"],
)

load("@rules_rust//rust:repositories.bzl", "rules_rust_dependencies", "rust_register_toolchains")

rules_rust_dependencies()

rust_register_toolchains(
    versions = ["1.74.1"],
)

load("@rules_rust//crate_universe:repositories.bzl", "crate_universe_dependencies")

crate_universe_dependencies()

load("//third-party/bazel:defs.bzl", "crate_repositories")

crate_repositories()

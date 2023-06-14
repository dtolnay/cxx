workspace(name = "cxx.rs")

load("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")

http_archive(
    name = "rules_rust",
    sha256 = "50ec4b84a7ec5370f5882d52f4a1e6b8a75de2f8dcc0a4403747b69b2c4ef5b1",
    urls = ["https://github.com/bazelbuild/rules_rust/releases/download/0.23.0/rules_rust-v0.23.0.tar.gz"],
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

load(
    "@io_bazel_rules_rust//rust:rust.bzl",
    _rust_binary = "rust_binary",
    _rust_library = "rust_library",
    _rust_test = "rust_test",
)

def rust_binary(edition = "2018", **kwargs):
    _rust_binary(edition = edition, **kwargs)

def third_party_rust_binary(rustc_flags = [], **kwargs):
    rustc_flags = rustc_flags + ["--cap-lints=allow"]
    rust_binary(rustc_flags = rustc_flags, **kwargs)

def rust_library(edition = "2018", **kwargs):
    _rust_library(edition = edition, **kwargs)

def third_party_rust_library(rustc_flags = [], **kwargs):
    rustc_flags = rustc_flags + ["--cap-lints=allow"]
    rust_library(rustc_flags = rustc_flags, **kwargs)

def rust_test(edition = "2018", **kwargs):
    _rust_test(edition = edition, **kwargs)

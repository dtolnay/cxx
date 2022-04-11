load("@rules_rust//cargo:cargo_build_script.bzl", "cargo_build_script")
load("@rules_rust//rust:defs.bzl", "rust_binary", "rust_library")
load("@third-party//:vendor.bzl", "vendored")

def third_party_glob(include):
    return vendored and native.glob(include)

def third_party_cargo_build_script(edition, rustc_flags = [], **kwargs):
    rustc_flags = rustc_flags + ["--cap-lints=allow"]
    cargo_build_script(edition = edition, rustc_flags = rustc_flags, **kwargs)

def third_party_rust_binary(edition, rustc_flags = [], **kwargs):
    rustc_flags = rustc_flags + ["--cap-lints=allow"]
    rust_binary(edition = edition, rustc_flags = rustc_flags, **kwargs)

def third_party_rust_library(edition, rustc_flags = [], **kwargs):
    rustc_flags = rustc_flags + ["--cap-lints=allow"]
    rust_library(edition = edition, rustc_flags = rustc_flags, **kwargs)

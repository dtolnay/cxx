load("//:build/rust.bzl", "rust_binary", "rust_library")

rust_library(
    name = "cxx",
    srcs = glob(["src/**/*.rs"]),
    data = ["src/gen/include/cxxbridge.h"],
    visibility = ["//visibility:public"],
    deps = [
        ":core_lib",
        ":cxxbridge_macro",
        "//third-party:anyhow",
        "//third-party:cc",
        "//third-party:codespan",
        "//third-party:codespan_reporting",
        "//third-party:link_cplusplus",
        "//third-party:proc_macro2",
        "//third-party:quote",
        "//third-party:syn",
        "//third-party:thiserror",
    ],
)

rust_binary(
    name = "codegen",
    srcs = glob(["cmd/src/**/*.rs"]),
    data = ["cmd/src/gen/include/cxxbridge.h"],
    visibility = ["//visibility:public"],
    deps = [
        "//third-party:anyhow",
        "//third-party:codespan",
        "//third-party:codespan_reporting",
        "//third-party:proc_macro2",
        "//third-party:quote",
        "//third-party:structopt",
        "//third-party:syn",
        "//third-party:thiserror",
    ],
)

cc_library(
    name = "core",
    hdrs = ["include/cxxbridge.h"],
    include_prefix = "cxxbridge",
    strip_include_prefix = "include",
    visibility = ["//visibility:public"],
)

cc_library(
    name = "core_lib",
    srcs = ["src/cxxbridge.cc"],
    hdrs = ["include/cxxbridge.h"],
)

rust_library(
    name = "cxxbridge_macro",
    srcs = glob(["macro/src/**"]),
    crate_type = "proc-macro",
    deps = [
        "//third-party:proc_macro2",
        "//third-party:quote",
        "//third-party:syn",
    ],
)

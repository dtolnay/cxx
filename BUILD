load("//tools/bazel:rust.bzl", "rust_binary", "rust_library")

rust_library(
    name = "cxx",
    srcs = glob(["src/**/*.rs"]),
    data = ["src/gen/include/cxx.h"],
    visibility = ["//visibility:public"],
    deps = [
        ":core-lib",
        ":cxxbridge-macro",
        "//third-party:anyhow",
        "//third-party:cc",
        "//third-party:codespan-reporting",
        "//third-party:link-cplusplus",
        "//third-party:proc-macro2",
        "//third-party:quote",
        "//third-party:syn",
        "//third-party:thiserror",
    ],
)

rust_binary(
    name = "codegen",
    srcs = glob(["cmd/src/**/*.rs"]),
    data = ["cmd/src/gen/include/cxx.h"],
    visibility = ["//visibility:public"],
    deps = [
        "//third-party:anyhow",
        "//third-party:codespan-reporting",
        "//third-party:proc-macro2",
        "//third-party:quote",
        "//third-party:structopt",
        "//third-party:syn",
        "//third-party:thiserror",
    ],
)

cc_library(
    name = "core",
    hdrs = ["include/cxx.h"],
    include_prefix = "rust",
    strip_include_prefix = "include",
    visibility = ["//visibility:public"],
)

cc_library(
    name = "core-lib",
    srcs = ["src/cxx.cc"],
    hdrs = ["include/cxx.h"],
)

rust_library(
    name = "cxxbridge-macro",
    srcs = glob(["macro/src/**"]),
    crate_type = "proc-macro",
    deps = [
        "//third-party:proc-macro2",
        "//third-party:quote",
        "//third-party:syn",
    ],
)

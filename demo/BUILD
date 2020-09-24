load("@rules_cc//cc:defs.bzl", "cc_library")
load("//tools/bazel:rust.bzl", "rust_binary")
load("//tools/bazel:rust_cxx_bridge.bzl", "rust_cxx_bridge")

rust_binary(
    name = "demo",
    srcs = glob(["src/**/*.rs"]),
    deps = [
        ":bridge",
        ":demo-sys",
        "//:cxx",
    ],
)

rust_cxx_bridge(
    name = "bridge",
    src = "src/main.rs",
    deps = [":demo-include"],
)

cc_library(
    name = "demo-sys",
    srcs = ["src/demo.cc"],
    copts = ["-std=c++14"],
    deps = [
        ":bridge/include",
        ":demo-include",
    ],
)

cc_library(
    name = "demo-include",
    hdrs = ["include/demo.h"],
    deps = ["//:core"],
)

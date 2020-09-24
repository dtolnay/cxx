load("@rules_cc//cc:defs.bzl", "cc_library")
load("//tools/bazel:rust.bzl", "rust_binary")

rust_binary(
    name = "demo",
    srcs = glob(["src/**/*.rs"]),
    deps = [
        ":bridge",
        ":demo-sys",
        "//:cxx",
    ],
)

cc_library(
    name = "bridge",
    srcs = [":bridge/source"],
    deps = [
        ":bridge/include",
        ":demo-include",
    ],
)

genrule(
    name = "bridge/header",
    srcs = ["src/main.rs"],
    outs = ["src/main.rs.h"],
    cmd = "$(location //:codegen) --header $< > $@",
    tools = ["//:codegen"],
)

genrule(
    name = "bridge/source",
    srcs = ["src/main.rs"],
    outs = ["src/main.rs.cc"],
    cmd = "$(location //:codegen) $< > $@",
    tools = ["//:codegen"],
)

cc_library(
    name = "bridge/include",
    hdrs = [":bridge/header"],
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

load("//tools/bazel:rust.bzl", "rust_binary", "rust_library")

rust_binary(
    name = "demo",
    srcs = glob(["src/**/*.rs"]),
    deps = [
        ":demo-sys",
        ":gen",
        "//:cxx",
    ],
)

cc_library(
    name = "gen",
    srcs = [":gen-source"],
    deps = [
        ":demo-include",
        ":include",
    ],
)

genrule(
    name = "gen-header",
    srcs = ["src/main.rs"],
    outs = ["main.rs.h"],
    cmd = "$(location //:codegen) --header $< > $@",
    tools = ["//:codegen"],
)

genrule(
    name = "gen-source",
    srcs = ["src/main.rs"],
    outs = ["generated.cc"],
    cmd = "$(location //:codegen) $< > $@",
    tools = ["//:codegen"],
)

cc_library(
    name = "include",
    hdrs = [":gen-header"],
    include_prefix = "demo/src",
)

cc_library(
    name = "demo-sys",
    srcs = ["src/demo.cc"],
    copts = ["-std=c++14"],
    deps = [
        ":demo-include",
        ":include",
    ],
)

cc_library(
    name = "demo-include",
    hdrs = ["include/demo.h"],
    deps = ["//:core"],
)

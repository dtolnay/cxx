load("//tools/bazel:rust.bzl", "rust_binary", "rust_library")

rust_binary(
    name = "demo-rs",
    srcs = glob(["src/**"]),
    deps = [
        ":gen",
        "//:cxx",
        "//demo-cxx",
    ],
)

cc_library(
    name = "gen",
    srcs = [":gen-source"],
    deps = [
        ":include",
        "//demo-cxx:include",
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
    include_prefix = "demo-rs/src",
    visibility = ["//visibility:public"],
)

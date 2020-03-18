load("//tools/bazel:rust.bzl", "rust_library", "rust_test")

rust_test(
    name = "test",
    srcs = ["test.rs"],
    deps = [":cxx_test_suite"],
)

rust_library(
    name = "cxx_test_suite",
    srcs = ["ffi/lib.rs"],
    deps = [
        ":impl",
        "//:cxx",
    ],
)

cc_library(
    name = "impl",
    srcs = [
        "ffi/tests.cc",
        ":gen-source",
    ],
    hdrs = ["ffi/tests.h"],
    deps = [
        ":include",
        "//:core",
    ],
)

genrule(
    name = "gen-header",
    srcs = ["ffi/lib.rs"],
    outs = ["lib.rs"],
    cmd = "$(location //:codegen) --header $< > $@",
    tools = ["//:codegen"],
)

genrule(
    name = "gen-source",
    srcs = ["ffi/lib.rs"],
    outs = ["generated.cc"],
    cmd = "$(location //:codegen) $< > $@",
    tools = ["//:codegen"],
)

cc_library(
    name = "include",
    hdrs = [":gen-header"],
    include_prefix = "tests/ffi",
)

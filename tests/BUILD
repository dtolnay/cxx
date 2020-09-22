load("@rules_cc//cc:defs.bzl", "cc_library")
load("//tools/bazel:rust.bzl", "rust_library", "rust_test")

rust_test(
    name = "test",
    size = "small",
    srcs = ["test.rs"],
    deps = [":cxx_test_suite"],
)

rust_library(
    name = "cxx_test_suite",
    srcs = [
        "ffi/lib.rs",
        "ffi/module.rs",
    ],
    deps = [
        ":impl",
        "//:cxx",
    ],
)

cc_library(
    name = "impl",
    srcs = [
        "ffi/tests.cc",
        ":bridge/source",
        ":module/source",
    ],
    hdrs = ["ffi/tests.h"],
    include_prefix = "cxx-test-suite",
    strip_include_prefix = "ffi",
    deps = [
        ":bridge/include",
        "//:core",
    ],
)

genrule(
    name = "bridge/header",
    srcs = ["ffi/lib.rs"],
    outs = ["ffi/lib.rs.h"],
    cmd = "$(location //:codegen) --header $< > $@",
    tools = ["//:codegen"],
)

genrule(
    name = "bridge/source",
    srcs = ["ffi/lib.rs"],
    outs = ["ffi/lib.rs.cc"],
    cmd = "$(location //:codegen) $< > $@",
    tools = ["//:codegen"],
)

cc_library(
    name = "bridge/include",
    hdrs = [":bridge/header"],
    include_prefix = "cxx-test-suite",
    strip_include_prefix = "ffi",
)

genrule(
    name = "module/source",
    srcs = ["ffi/module.rs"],
    outs = ["ffi/module.rs.cc"],
    cmd = "$(location //:codegen) $< > $@",
    tools = ["//:codegen"],
)

load("//tools/bazel:rust.bzl", "rust_library", "rust_test")

rust_test(
    name = "test",
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
        ":gen-lib-source",
        ":gen-module-source",
    ],
    hdrs = ["ffi/tests.h"],
    deps = [
        ":lib-include",
        "//:core",
    ],
)

genrule(
    name = "gen-lib-header",
    srcs = ["ffi/lib.rs"],
    outs = ["lib.rs.h"],
    cmd = "$(location //:codegen) --header $< > $@",
    tools = ["//:codegen"],
)

genrule(
    name = "gen-lib-source",
    srcs = ["ffi/lib.rs"],
    outs = ["lib.rs.cc"],
    cmd = "$(location //:codegen) $< > $@",
    tools = ["//:codegen"],
)

cc_library(
    name = "lib-include",
    hdrs = [":gen-lib-header"],
    include_prefix = "tests/ffi",
)

genrule(
    name = "gen-module-source",
    srcs = ["ffi/module.rs"],
    outs = ["module.rs.cc"],
    cmd = "$(location //:codegen) $< > $@",
    tools = ["//:codegen"],
)

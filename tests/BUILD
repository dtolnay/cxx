load("@rules_cc//cc:defs.bzl", "cc_library")
load("//tools/bazel:rust.bzl", "rust_library", "rust_test")
load("//tools/bazel:rust_cxx_bridge.bzl", "rust_cxx_bridge")

rust_test(
    name = "test",
    size = "small",
    srcs = ["test.rs"],
    deps = [":cxx_test_suite"],
)

rust_library(
    name = "cxx_test_suite",
    srcs = [
        "ffi/alias.rs",
        "ffi/lib.rs",
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
        ":alias/source",
        ":bridge/source",
    ],
    hdrs = ["ffi/tests.h"],
    include_prefix = "cxx-test-suite",
    strip_include_prefix = "ffi",
    deps = [
        ":bridge/include",
        "//:core",
    ],
)

rust_cxx_bridge(
    name = "bridge",
    src = "ffi/lib.rs",
    include_prefix = "cxx-test-suite",
    strip_include_prefix = "ffi",
    deps = [":impl"],
)

rust_cxx_bridge(
    name = "alias",
    src = "ffi/alias.rs",
    include_prefix = "cxx-test-suite",
    strip_include_prefix = "ffi",
    deps = [":impl"],
)

load("//tools/bazel:cxxbridge_cc_library.bzl", "cxxbridge_cc_library")
load("//tools/bazel:rust.bzl", "rust_library", "rust_test")

rust_test(
    name = "test",
    srcs = ["test.rs"],
    deps = [":cxx_test_suite"],
)

bridge_srcs = [
    "ffi/alias.rs",
    "ffi/alias2.rs",
    "ffi/lib.rs",
]

rust_library(
    name = "cxx_test_suite",
    srcs = bridge_srcs,
    deps = [
        ":impl",
        "//:cxx",
    ],
)

cxxbridge_cc_library(
    name = "impl",
    crate_name = "cxx-test-suite",
    bridge_srcs = bridge_srcs,
    srcs = ["ffi/tests.cc"],
    hdrs = ["ffi/tests.h"],
    strip_include_prefix = "ffi",
    deps = ["//:core"],
)

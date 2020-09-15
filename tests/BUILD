load("//tools/bazel:rust.bzl", "rust_library", "rust_test")

rust_test(
    name = "test",
    srcs = ["test.rs"],
    deps = [":cxx_test_suite"],
)

rust_library(
    name = "cxx_test_suite",
    srcs = [
        "ffi/alias.rs",
        "ffi/alias2.rs",
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
        ":gen-alias-source",
        ":gen-alias2-source",
        ":gen-lib-source",
    ],
    hdrs = ["ffi/tests.h"],
    include_prefix = "cxx-test-suite",
    strip_include_prefix = "ffi",
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
    include_prefix = "cxx-test-suite",
)

genrule(
    name = "gen-alias-source",
    srcs = ["ffi/alias.rs"],
    outs = ["alias.rs.cc"],
    cmd = "$(location //:codegen) $< > $@",
    tools = ["//:codegen"],
)

genrule(
    name = "gen-alias2-source",
    srcs = ["ffi/alias2.rs"],
    outs = ["alias2.rs.cc"],
    cmd = "$(location //:codegen) $< > $@",
    tools = ["//:codegen"],
)

load("@bazel_skylib//rules:run_binary.bzl", "run_binary")
load("@rules_cc//cc:defs.bzl", "cc_library")

def rust_cxx_bridge(
        name,
        src,
        include_prefix = None,
        strip_include_prefix = None,
        deps = []):
    run_binary(
        name = "%s/header" % name,
        srcs = [src],
        outs = [src + ".h"],
        args = [
            "$(location %s)" % src,
            "-o",
            "$(location %s.h)" % src,
            "--header",
        ],
        tool = "//:codegen",
    )

    run_binary(
        name = "%s/source" % name,
        srcs = [src],
        outs = [src + ".cc"],
        args = [
            "$(location %s)" % src,
            "-o",
            "$(location %s.cc)" % src,
        ],
        tool = "//:codegen",
    )

    cc_library(
        name = name,
        srcs = [":%s/source" % name],
        deps = deps + [":%s/include" % name],
    )

    cc_library(
        name = "%s/include" % name,
        hdrs = [":%s/header" % name],
        include_prefix = include_prefix,
        strip_include_prefix = strip_include_prefix,
    )

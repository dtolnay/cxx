load("@rules_cc//cc:defs.bzl", "cc_library")

def rust_cxx_bridge(
        name,
        src,
        include_prefix = None,
        strip_include_prefix = None,
        deps = []):
    native.genrule(
        name = "%s/header" % name,
        srcs = [src],
        outs = [src + ".h"],
        cmd = "$(location //:codegen) --header $< > $@",
        tools = ["//:codegen"],
    )

    native.genrule(
        name = "%s/source" % name,
        srcs = [src],
        outs = [src + ".cc"],
        cmd = "$(location //:codegen) $< > $@",
        tools = ["//:codegen"],
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

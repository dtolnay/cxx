load("//tools/buck:genrule.bzl", "genrule")

def rust_cxx_bridge(name, src, deps = []):
    genrule(
        name = "%s/header" % name,
        srcs = [src],
        out = src + ".h",
        cmd = "$(exe //:codegen) --header ${SRCS} > ${OUT}",
        type = "cxxbridge",
    )

    genrule(
        name = "%s/source" % name,
        srcs = [src],
        out = src + ".cc",
        cmd = "$(exe //:codegen) ${SRCS} > ${OUT}",
        type = "cxxbridge",
    )

    cxx_library(
        name = name,
        srcs = [":%s/source" % name],
        preferred_linkage = "static",
        deps = deps + [":%s/include" % name],
    )

    cxx_library(
        name = "%s/include" % name,
        exported_headers = [":%s/header" % name],
    )

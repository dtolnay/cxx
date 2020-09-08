rust_library(
    name = "cxx",
    srcs = glob(["src/**"]),
    visibility = ["PUBLIC"],
    deps = [
        ":core",
        ":macro",
    ],
)

rust_binary(
    name = "codegen",
    srcs = glob(["gen/cmd/src/**"]),
    crate = "cxxbridge",
    visibility = ["PUBLIC"],
    deps = [
        "//third-party:clap",
        "//third-party:codespan-reporting",
        "//third-party:proc-macro2",
        "//third-party:quote",
        "//third-party:syn",
    ],
)

cxx_library(
    name = "core",
    srcs = ["src/cxx.cc"],
    visibility = ["PUBLIC"],
    header_namespace = "rust",
    exported_headers = {
        "cxx.h": "include/cxx.h",
    },
    exported_linker_flags = ["-lstdc++"],
)

rust_library(
    name = "macro",
    srcs = glob(["macro/src/**"]),
    proc_macro = True,
    crate = "cxxbridge_macro",
    deps = [
        "//third-party:proc-macro2",
        "//third-party:quote",
        "//third-party:syn",
    ],
)

rust_library(
    name = "build",
    srcs = glob(["gen/build/src/**"]),
    visibility = ["PUBLIC"],
    deps = [
        "//third-party:cc",
        "//third-party:codespan-reporting",
        "//third-party:proc-macro2",
        "//third-party:quote",
        "//third-party:syn",
    ],
)

rust_library(
    name = "lib",
    srcs = glob(["gen/lib/src/**"]),
    visibility = ["PUBLIC"],
    deps = [
        "//third-party:cc",
        "//third-party:codespan-reporting",
        "//third-party:proc-macro2",
        "//third-party:quote",
        "//third-party:syn",
    ],
)

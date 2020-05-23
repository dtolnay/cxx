load(
    "//tools/bazel:rust.bzl",
    rust_binary = "third_party_rust_binary",
    rust_library = "third_party_rust_library",
)
load("@bazel_tools//tools/build_defs/pkg:pkg.bzl", "pkg_tar")

rust_library(
    name = "anyhow",
    srcs = glob(["vendor/anyhow-1.0.31/src/**"]),
    crate_features = ["std"],
    visibility = ["//visibility:public"],
)

rust_library(
    name = "bitflags",
    srcs = glob(["vendor/bitflags-1.2.1/src/**"]),
)

rust_library(
    name = "cc",
    srcs = glob(["vendor/cc-1.0.54/src/**"]),
    visibility = ["//visibility:public"],
)

rust_library(
    name = "clap",
    srcs = glob(["vendor/clap-2.33.1/src/**"]),
    edition = "2015",
    deps = [
        ":bitflags",
        ":textwrap",
        ":unicode-width",
    ],
)

rust_library(
    name = "codespan-reporting",
    srcs = glob(["vendor/codespan-reporting-0.9.4/src/**"]),
    visibility = ["//visibility:public"],
    deps = [
        ":termcolor",
        ":unicode-width",
    ],
)

rust_library(
    name = "heck",
    srcs = glob(["vendor/heck-0.3.1/src/**"]),
    edition = "2015",
    deps = [":unicode-segmentation"],
)

rust_library(
    name = "lazy_static",
    srcs = glob(["vendor/lazy_static-1.4.0/src/**"]),
)

rust_library(
    name = "link-cplusplus",
    srcs = glob(["vendor/link-cplusplus-1.0.1/src/**"]),
    visibility = ["//visibility:public"],
)

rust_library(
    name = "proc-macro-error",
    srcs = glob(["vendor/proc-macro-error-1.0.2/src/**"]),
    rustc_flags = ["--cfg=use_fallback"],
    deps = [
        ":proc-macro-error-attr",
        ":proc-macro2",
        ":quote",
        ":syn",
    ],
)

rust_library(
    name = "proc-macro-error-attr",
    srcs = glob(["vendor/proc-macro-error-attr-1.0.2/src/**"]),
    crate_type = "proc-macro",
    deps = [
        ":proc-macro2",
        ":quote",
        ":syn",
        ":syn-mid",
    ],
)

rust_library(
    name = "proc-macro2",
    srcs = glob(["vendor/proc-macro2-1.0.17/src/**"]),
    crate_features = [
        "proc-macro",
        "span-locations",
    ],
    rustc_flags = [
        "--cfg=span_locations",
        "--cfg=use_proc_macro",
        "--cfg=wrap_proc_macro",
    ],
    visibility = ["//visibility:public"],
    deps = [":unicode-xid"],
)

rust_library(
    name = "quote",
    srcs = glob(["vendor/quote-1.0.6/src/**"]),
    crate_features = ["proc-macro"],
    visibility = ["//visibility:public"],
    deps = [":proc-macro2"],
)

rust_library(
    name = "structopt",
    srcs = glob(["vendor/structopt-0.3.14/src/**"]),
    visibility = ["//visibility:public"],
    deps = [
        ":clap",
        ":lazy_static",
        ":structopt-derive",
    ],
)

rust_library(
    name = "structopt-derive",
    srcs = glob(["vendor/structopt-derive-0.4.7/src/**"]),
    crate_type = "proc-macro",
    deps = [
        ":heck",
        ":proc-macro-error",
        ":proc-macro2",
        ":quote",
        ":syn",
    ],
)

rust_library(
    name = "syn",
    srcs = glob(["vendor/syn-1.0.23/src/**"]),
    crate_features = [
        "clone-impls",
        "derive",
        "full",
        "parsing",
        "printing",
        "proc-macro",
    ],
    visibility = ["//visibility:public"],
    deps = [
        ":proc-macro2",
        ":quote",
        ":unicode-xid",
    ],
)

rust_library(
    name = "syn-mid",
    srcs = glob(["vendor/syn-mid-0.5.0/src/**"]),
    deps = [
        ":proc-macro2",
        ":quote",
        ":syn",
    ],
)

rust_library(
    name = "termcolor",
    srcs = glob(["vendor/termcolor-1.1.0/src/**"]),
)

rust_library(
    name = "textwrap",
    srcs = glob(["vendor/textwrap-0.11.0/src/**"]),
    deps = [":unicode-width"],
)

rust_library(
    name = "unicode-segmentation",
    srcs = glob(["vendor/unicode-segmentation-1.6.0/src/**"]),
    edition = "2015",
)

rust_library(
    name = "unicode-width",
    srcs = glob(["vendor/unicode-width-0.1.7/src/**"]),
)

rust_library(
    name = "unicode-xid",
    srcs = glob(["vendor/unicode-xid-0.2.0/src/**"]),
)

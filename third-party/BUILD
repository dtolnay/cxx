load(
    "//tools/bazel:rust.bzl",
    rust_binary = "third_party_rust_binary",
    rust_library = "third_party_rust_library",
)
load("@bazel_tools//tools/build_defs/pkg:pkg.bzl", "pkg_tar")

rust_library(
    name = "anyhow",
    srcs = glob(["vendor/anyhow-1.0.32/src/**"]),
    crate_features = ["std"],
    visibility = ["//visibility:public"],
)

rust_library(
    name = "atty",
    srcs = glob(["vendor/atty-0.2.14/src/**"]),
    deps = [":libc"],
)

rust_library(
    name = "bitflags",
    srcs = glob(["vendor/bitflags-1.2.1/src/**"]),
)

rust_library(
    name = "cc",
    srcs = glob(["vendor/cc-1.0.59/src/**"]),
    visibility = ["//visibility:public"],
)

rust_library(
    name = "clap",
    srcs = glob(["vendor/clap-3.0.0-beta.1/src/**"]),
    crate_features = [
        "cargo",
        "color",
        "std",
        "suggestions",
    ],
    visibility = ["//visibility:public"],
    deps = [
        ":atty",
        ":bitflags",
        ":indexmap",
        ":lazy_static",
        ":os_str_bytes",
        ":strsim",
        ":termcolor",
        ":textwrap",
        ":unicode-width",
        ":vec_map",
    ],
)

rust_library(
    name = "codespan-reporting",
    srcs = glob(["vendor/codespan-reporting-0.9.5/src/**"]),
    visibility = ["//visibility:public"],
    deps = [
        ":termcolor",
        ":unicode-width",
    ],
)

rust_library(
    name = "hashbrown",
    srcs = glob(["vendor/hashbrown-0.8.2/src/**"]),
    crate_features = ["raw"],
)

rust_library(
    name = "indexmap",
    srcs = glob(["vendor/indexmap-1.5.1/src/**"]),
    rustc_flags = ["--cfg=has_std"],
    deps = [":hashbrown"],
)

rust_library(
    name = "lazy_static",
    srcs = glob(["vendor/lazy_static-1.4.0/src/**"]),
)

rust_library(
    name = "libc",
    srcs = glob(["vendor/libc-0.2.76/src/**"]),
    crate_features = ["std"],
    edition = "2015",
    rustc_flags = [
        "--cfg=libc_packedN",
        "--cfg=libc_cfg_target_vendor",
        "--cfg=libc_core_cvoid",
        "--cfg=libc_align",
        "--cfg=libc_const_size_of",
        "--cfg=libc_union",
        "--cfg=libc_priv_mod_use",
    ],
)

rust_library(
    name = "link-cplusplus",
    srcs = glob(["vendor/link-cplusplus-1.0.2/src/**"]),
    visibility = ["//visibility:public"],
)

rust_library(
    name = "os_str_bytes",
    srcs = glob(["vendor/os_str_bytes-2.3.2/src/**"]),
    crate_features = ["raw"],
)

rust_library(
    name = "proc-macro2",
    srcs = glob(["vendor/proc-macro2-1.0.19/src/**"]),
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
    srcs = glob(["vendor/quote-1.0.7/src/**"]),
    crate_features = ["proc-macro"],
    visibility = ["//visibility:public"],
    deps = [":proc-macro2"],
)

rust_library(
    name = "strsim",
    srcs = glob(["vendor/strsim-0.10.0/src/**"]),
)

rust_library(
    name = "syn",
    srcs = glob(["vendor/syn-1.0.39/src/**"]),
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
    name = "termcolor",
    srcs = glob(["vendor/termcolor-1.1.0/src/**"]),
)

rust_library(
    name = "textwrap",
    srcs = glob(["vendor/textwrap-0.12.1/src/**"]),
    deps = [":unicode-width"],
)

rust_library(
    name = "unicode-width",
    srcs = glob(["vendor/unicode-width-0.1.8/src/**"]),
)

rust_library(
    name = "unicode-xid",
    srcs = glob(["vendor/unicode-xid-0.2.1/src/**"]),
)

rust_library(
    name = "vec_map",
    srcs = glob(["vendor/vec_map-0.8.2/src/**"]),
)

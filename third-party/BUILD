load(
    "//tools/bazel:third_party.bzl",
    cargo_build_script = "third_party_cargo_build_script",
    glob = "third_party_glob",
    rust_library = "third_party_rust_library",
)

rust_library(
    name = "bitflags",
    srcs = glob(["vendor/bitflags-1.3.2/src/**"]),
)

rust_library(
    name = "cc",
    srcs = glob(["vendor/cc-1.0.72/src/**"]),
    visibility = ["//visibility:public"],
)

rust_library(
    name = "clap",
    srcs = glob(["vendor/clap-3.0.10/src/**"]),
    crate_features = ["std"],
    visibility = ["//visibility:public"],
    deps = [
        ":bitflags",
        ":indexmap",
        ":os_str_bytes",
        ":textwrap",
    ],
)

rust_library(
    name = "codespan-reporting",
    srcs = glob(["vendor/codespan-reporting-0.11.1/src/**"]),
    visibility = ["//visibility:public"],
    deps = [
        ":termcolor",
        ":unicode-width",
    ],
)

rust_library(
    name = "hashbrown",
    srcs = glob(["vendor/hashbrown-0.11.2/src/**"]),
    crate_features = ["raw"],
)

rust_library(
    name = "indexmap",
    srcs = glob(["vendor/indexmap-1.8.0/src/**"]),
    crate_features = ["std"],
    rustc_flags = ["--cfg=has_std"],
    deps = [":hashbrown"],
)

rust_library(
    name = "memchr",
    srcs = glob(["vendor/memchr-2.4.1/src/**/*.rs"]),
    crate_features = ["std"],
)

rust_library(
    name = "once_cell",
    srcs = glob(["vendor/once_cell-1.9.0/src/**"]),
    crate_features = [
        "alloc",
        "std",
    ],
    visibility = ["//visibility:public"],
)

rust_library(
    name = "os_str_bytes",
    srcs = glob(["vendor/os_str_bytes-6.0.0/src/**"]),
    crate_features = [
        "memchr",
        "raw_os_str",
    ],
    deps = [":memchr"],
)

rust_library(
    name = "proc-macro2",
    srcs = glob(["vendor/proc-macro2-1.0.36/src/**"]),
    crate_features = [
        "proc-macro",
        "span-locations",
    ],
    visibility = ["//visibility:public"],
    deps = [
        ":proc-macro2@build",
        ":unicode-xid",
    ],
)

cargo_build_script(
    name = "proc-macro2@build",
    srcs = ["vendor/proc-macro2-1.0.36/build.rs"],
    crate_features = [
        "proc-macro",
        "span-locations",
    ],
    crate_name = "build",
)

rust_library(
    name = "quote",
    srcs = glob(["vendor/quote-1.0.14/src/**"]),
    crate_features = ["proc-macro"],
    visibility = ["//visibility:public"],
    deps = [":proc-macro2"],
)

rust_library(
    name = "scratch",
    srcs = glob(["vendor/scratch-1.0.1/src/**"]),
    rustc_env = {"OUT_DIR": ""},
    visibility = ["//visibility:public"],
)

rust_library(
    name = "syn",
    srcs = glob(["vendor/syn-1.0.85/src/**"]),
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
        ":syn@build",
        ":unicode-xid",
    ],
)

cargo_build_script(
    name = "syn@build",
    srcs = ["vendor/syn-1.0.85/build.rs"],
    crate_features = [
        "clone-impls",
        "derive",
        "full",
        "parsing",
        "printing",
        "proc-macro",
    ],
    crate_name = "build",
)

rust_library(
    name = "termcolor",
    srcs = glob(["vendor/termcolor-1.1.2/src/**"]),
)

rust_library(
    name = "textwrap",
    srcs = glob(["vendor/textwrap-0.14.2/src/**"]),
    deps = [":unicode-width"],
)

rust_library(
    name = "unicode-width",
    srcs = glob(["vendor/unicode-width-0.1.9/src/**"]),
)

rust_library(
    name = "unicode-xid",
    srcs = glob(["vendor/unicode-xid-0.2.2/src/**"]),
)

load(
    "//tools/bazel:third_party.bzl",
    cargo_build_script = "third_party_cargo_build_script",
    glob = "third_party_glob",
    rust_library = "third_party_rust_library",
)

rust_library(
    name = "bitflags",
    srcs = glob(["vendor/bitflags-1.3.2/src/**/*.rs"]),
    edition = "2018",
)

rust_library(
    name = "cc",
    srcs = glob(["vendor/cc-1.0.77/src/**/*.rs"]),
    edition = "2018",
    visibility = ["//visibility:public"],
)

rust_library(
    name = "clap",
    srcs = glob(["vendor/clap-4.0.29/src/**/*.rs"]) + ["vendor/clap-4.0.29/examples/demo.rs"],
    crate_features = [
        "error-context",
        "help",
        "std",
        "usage",
    ],
    data = ["vendor/clap-4.0.29/examples/demo.md"],
    edition = "2021",
    visibility = ["//visibility:public"],
    deps = [
        ":bitflags",
        ":clap_lex",
    ],
)

rust_library(
    name = "clap_lex",
    srcs = glob(["vendor/clap_lex-0.3.0/src/**/*.rs"]),
    edition = "2021",
    deps = [
        ":os_str_bytes",
    ],
)

rust_library(
    name = "codespan-reporting",
    srcs = glob(["vendor/codespan-reporting-0.11.1/src/**/*.rs"]),
    edition = "2018",
    visibility = ["//visibility:public"],
    deps = [
        ":termcolor",
        ":unicode-width",
    ],
)

rust_library(
    name = "once_cell",
    srcs = glob(["vendor/once_cell-1.16.0/src/**/*.rs"]),
    crate_features = [
        "alloc",
        "default",
        "race",
        "std",
    ],
    edition = "2021",
    visibility = ["//visibility:public"],
)

rust_library(
    name = "os_str_bytes",
    srcs = glob(["vendor/os_str_bytes-6.4.1/src/**/*.rs"]),
    crate_features = [
        "raw_os_str",
    ],
    edition = "2021",
)

rust_library(
    name = "proc-macro2",
    srcs = glob(["vendor/proc-macro2-1.0.47/src/**/*.rs"]),
    crate_features = [
        "default",
        "proc-macro",
        "span-locations",
    ],
    edition = "2018",
    visibility = ["//visibility:public"],
    deps = [
        ":proc-macro2@build",
        ":unicode-ident",
    ],
)

cargo_build_script(
    name = "proc-macro2@build",
    srcs = ["vendor/proc-macro2-1.0.47/build.rs"],
    crate_features = [
        "default",
        "proc-macro",
        "span-locations",
    ],
    crate_name = "build",
    edition = "2018",
)

rust_library(
    name = "quote",
    srcs = glob(["vendor/quote-1.0.21/src/**/*.rs"]),
    crate_features = [
        "default",
        "proc-macro",
    ],
    edition = "2018",
    visibility = ["//visibility:public"],
    deps = [
        ":proc-macro2",
        ":quote@build",
    ],
)

cargo_build_script(
    name = "quote@build",
    srcs = ["vendor/quote-1.0.21/build.rs"],
    crate_features = [
        "default",
        "proc-macro",
    ],
    crate_name = "build",
    edition = "2018",
)

rust_library(
    name = "scratch",
    srcs = glob(["vendor/scratch-1.0.2/src/**/*.rs"]),
    edition = "2015",
    rustc_env = {"OUT_DIR": ""},
    visibility = ["//visibility:public"],
)

rust_library(
    name = "syn",
    srcs = glob(["vendor/syn-1.0.105/src/**/*.rs"]),
    crate_features = [
        "clone-impls",
        "default",
        "derive",
        "full",
        "parsing",
        "printing",
        "proc-macro",
        "quote",
    ],
    edition = "2018",
    visibility = ["//visibility:public"],
    deps = [
        ":proc-macro2",
        ":quote",
        ":syn@build",
        ":unicode-ident",
    ],
)

cargo_build_script(
    name = "syn@build",
    srcs = ["vendor/syn-1.0.105/build.rs"],
    crate_features = [
        "clone-impls",
        "default",
        "derive",
        "full",
        "parsing",
        "printing",
        "proc-macro",
        "quote",
    ],
    crate_name = "build",
    edition = "2018",
)

rust_library(
    name = "termcolor",
    srcs = glob(["vendor/termcolor-1.1.3/src/**/*.rs"]),
    edition = "2018",
)

rust_library(
    name = "unicode-ident",
    srcs = glob(["vendor/unicode-ident-1.0.5/src/**/*.rs"]),
    edition = "2018",
)

rust_library(
    name = "unicode-width",
    srcs = glob(["vendor/unicode-width-0.1.10/src/**/*.rs"]),
    crate_features = [
        "default",
    ],
    edition = "2015",
)

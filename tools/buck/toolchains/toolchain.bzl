load(
    "@prelude//cxx:cxx_toolchain_types.bzl",
    "BinaryUtilitiesInfo",
    "CCompilerInfo",
    "CxxCompilerInfo",
    "CxxPlatformInfo",
    "CxxToolchainInfo",
    "LinkerInfo",
)
load("@prelude//cxx:headers.bzl", "HeaderMode")
load("@prelude//linking:link_info.bzl", "LinkStyle")
load("@prelude//python_bootstrap:python_bootstrap.bzl", "PythonBootstrapToolchainInfo")
load("@prelude//rust:rust_toolchain.bzl", "RustPlatformInfo", "RustToolchainInfo")

DEFAULT_MAKE_COMP_DB = "prelude//cxx/tools:make_comp_db"

def _cxx_toolchain(ctx):
    """
    A very simple toolchain that is hardcoded to the current environment.
    """
    return [
        DefaultInfo(),
        CxxToolchainInfo(
            binary_utilities_info = BinaryUtilitiesInfo(
                nm = RunInfo(args = ["nm"]),
                ranlib = RunInfo(args = ["raninfo"]),
                strip = RunInfo(args = ["strip"]),
            ),
            c_compiler_info = CCompilerInfo(),
            cxx_compiler_info = CxxCompilerInfo(
                compiler = RunInfo(args = ["clang++"]),
                compiler_type = "clang",
            ),
            header_mode = HeaderMode("symlink_tree_only"),
            linker_info = LinkerInfo(
                archive_objects_locally = True,
                archiver = RunInfo(args = ["ar", "rcs"]),
                archiver_type = "gnu",
                binary_extension = "",
                link_binaries_locally = True,
                link_style = LinkStyle(ctx.attrs.link_style),
                link_weight = 1,
                linker = RunInfo(args = ["g++"]),
                linker_flags = ["-lstdc++"],
                mk_shlib_intf = ctx.attrs.make_shlib_intf,
                object_file_extension = "o",
                shared_library_name_format = "lib{}.so",
                shared_library_versioned_name_format = "lib{}.so.{}",
                static_library_extension = "a",
                type = "gnu",
                use_archiver_flags = False,
            ),
            mk_comp_db = ctx.attrs.make_comp_db,
        ),
        CxxPlatformInfo(name = "x86_64"),
    ]

cxx_toolchain = rule(
    impl = _cxx_toolchain,
    attrs = {
        "link_style": attrs.string(default = "static"),
        "make_comp_db": attrs.dep(providers = [RunInfo], default = DEFAULT_MAKE_COMP_DB),
        "make_shlib_intf": attrs.dep(providers = [RunInfo], default = DEFAULT_MAKE_COMP_DB),
    },
    is_toolchain_rule = True,
)

def _python_bootstrap_toolchain(_ctx):
    return [
        DefaultInfo(),
        PythonBootstrapToolchainInfo(interpreter = "python3"),
    ]

python_bootstrap_toolchain = rule(
    impl = _python_bootstrap_toolchain,
    attrs = {},
    is_toolchain_rule = True,
)

def _rust_toolchain(ctx):
    return [
        DefaultInfo(),
        RustToolchainInfo(
            clippy_driver = RunInfo(args = ["clippy-driver"]),
            compiler = RunInfo(args = ["rustc"]),
            default_edition = None,
            failure_filter = False,
            failure_filter_action = ctx.attrs.failure_filter_action[RunInfo],
            rustc_action = ctx.attrs.rustc_action[RunInfo],
            rustc_flags = ["-Clink-arg=-fuse-ld=lld"],
            rustc_target_triple = "x86_64-unknown-linux-gnu",
            rustdoc = RunInfo(args = ["rustdoc"]),
            rustdoc_flags = ["-Zunstable-options"],  # doc builds use unstable '--extern-html-root-url'
        ),
        RustPlatformInfo(name = "x86_64"),
    ]

rust_toolchain = rule(
    impl = _rust_toolchain,
    attrs = {
        "failure_filter_action": attrs.dep(providers = [RunInfo], default = "prelude//rust/tools:failure_filter_action"),
        "rustc_action": attrs.dep(providers = [RunInfo], default = "prelude//rust/tools:rustc_action"),
    },
    is_toolchain_rule = True,
)

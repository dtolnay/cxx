load(
    "@prelude//cxx:cxx_toolchain_types.bzl",
    "BinaryUtilitiesInfo",
    "CCompilerInfo",
    "CxxCompilerInfo",
    "CxxPlatformInfo",
    "CxxToolchainInfo",
    "LinkerInfo",
    "PicBehavior",
)
load("@prelude//cxx:headers.bzl", "HeaderMode")
load("@prelude//cxx:linker.bzl", "is_pdb_generated")
load("@prelude//linking:link_info.bzl", "LinkStyle")
load("@prelude//linking:lto.bzl", "LtoMode")
load("@prelude//utils:cmd_script.bzl", "ScriptOs", "cmd_script")
load("@toolchains//msvc:tools.bzl", "VisualStudio")

def _system_cxx_toolchain_impl(ctx):
    archiver_args = ["ar", "rcs"]
    archiver_type = "gnu"
    asm_compiler = ctx.attrs.compiler
    asm_compiler_type = ctx.attrs.compiler_type
    compiler = ctx.attrs.compiler
    cxx_compiler = ctx.attrs.cxx_compiler
    linker = ctx.attrs.linker
    linker_type = "gnu"
    pic_behavior = PicBehavior("supported")
    binary_extension = ""
    object_file_extension = "o"
    static_library_extension = "a"
    shared_library_name_format = "lib{}.so"
    shared_library_versioned_name_format = "lib{}.so.{}"
    additional_linker_flags = []
    if host_info().os.is_macos:
        linker_type = "darwin"
        pic_behavior = PicBehavior("always_enabled")
    elif host_info().os.is_windows:
        msvc_tools = ctx.attrs.msvc_tools[VisualStudio]
        archiver_args = [msvc_tools.lib_exe]
        archiver_type = "windows"
        asm_compiler = msvc_tools.ml64_exe
        asm_compiler_type = "windows_ml64"
        compiler = _windows_compiler_wrapper(ctx)
        cxx_compiler = compiler
        linker = _windows_linker_wrapper(ctx)
        linker_type = "windows"
        binary_extension = "exe"
        object_file_extension = "obj"
        static_library_extension = "lib"
        shared_library_name_format = "{}.dll"
        shared_library_versioned_name_format = "{}.dll"
        additional_linker_flags = ["msvcrt.lib"]
        pic_behavior = PicBehavior("not_supported")
    elif ctx.attrs.linker == "g++" or ctx.attrs.cxx_compiler == "g++":
        pass
    else:
        additional_linker_flags = ["-fuse-ld=lld"]

    return [
        DefaultInfo(),
        CxxToolchainInfo(
            mk_comp_db = ctx.attrs.make_comp_db,
            linker_info = LinkerInfo(
                linker = RunInfo(args = linker),
                linker_flags = additional_linker_flags + ctx.attrs.link_flags,
                archiver = RunInfo(args = archiver_args),
                archiver_type = archiver_type,
                generate_linker_maps = False,
                lto_mode = LtoMode("none"),
                type = linker_type,
                link_binaries_locally = True,
                archive_objects_locally = True,
                use_archiver_flags = False,
                static_dep_runtime_ld_flags = [],
                static_pic_dep_runtime_ld_flags = [],
                shared_dep_runtime_ld_flags = [],
                independent_shlib_interface_linker_flags = [],
                shlib_interfaces = "disabled",
                link_style = LinkStyle(ctx.attrs.link_style),
                link_weight = 1,
                binary_extension = binary_extension,
                object_file_extension = object_file_extension,
                shared_library_name_format = shared_library_name_format,
                shared_library_versioned_name_format = shared_library_versioned_name_format,
                static_library_extension = static_library_extension,
                force_full_hybrid_if_capable = False,
                is_pdb_generated = is_pdb_generated(linker_type, ctx.attrs.link_flags),
            ),
            bolt_enabled = False,
            binary_utilities_info = BinaryUtilitiesInfo(
                nm = RunInfo(args = ["nm"]),
                objcopy = RunInfo(args = ["objcopy"]),
                ranlib = RunInfo(args = ["ranlib"]),
                strip = RunInfo(args = ["strip"]),
                dwp = None,
                bolt_msdk = None,
            ),
            cxx_compiler_info = CxxCompilerInfo(
                compiler = RunInfo(args = [cxx_compiler]),
                preprocessor_flags = [],
                compiler_flags = ctx.attrs.cxx_flags,
                compiler_type = ctx.attrs.compiler_type,
            ),
            c_compiler_info = CCompilerInfo(
                compiler = RunInfo(args = [compiler]),
                preprocessor_flags = [],
                compiler_flags = ctx.attrs.c_flags,
                compiler_type = ctx.attrs.compiler_type,
            ),
            as_compiler_info = CCompilerInfo(
                compiler = RunInfo(args = [compiler]),
                compiler_type = ctx.attrs.compiler_type,
            ),
            asm_compiler_info = CCompilerInfo(
                compiler = RunInfo(args = [asm_compiler]),
                compiler_type = asm_compiler_type,
            ),
            header_mode = HeaderMode("symlink_tree_only"),
            cpp_dep_tracking_mode = ctx.attrs.cpp_dep_tracking_mode,
            pic_behavior = pic_behavior,
        ),
        CxxPlatformInfo(name = "x86_64"),
    ]

def _windows_linker_wrapper(ctx: AnalysisContext) -> cmd_args:
    # Linkers pretty much all support @file.txt argument syntax to insert
    # arguments from the given text file, usually formatted one argument per
    # line.
    #
    # - GNU ld: https://gcc.gnu.org/onlinedocs/gcc/Overall-Options.html
    # - lld is command line compatible with GNU ld
    # - MSVC link.exe: https://learn.microsoft.com/en-us/cpp/build/reference/linking?view=msvc-170#link-command-files
    #
    # However, there is inconsistency in whether they support nesting of @file
    # arguments inside of another @file.
    #
    # We wrap the linker to flatten @file arguments down to 1 level of nesting.
    return cmd_script(
        ctx = ctx,
        name = "windows_linker",
        cmd = cmd_args(
            ctx.attrs.linker_wrapper[RunInfo],
            ctx.attrs.linker,
        ),
        os = ScriptOs("windows"),
    )

def _windows_compiler_wrapper(ctx: AnalysisContext) -> cmd_args:
    # The wrapper is needed to dynamically find compiler location and
    # Windows SDK to add necessary includes.
    if ctx.attrs.compiler == "cl.exe":
        return cmd_script(
            ctx = ctx,
            name = "windows_compiler",
            cmd = cmd_args(
                ctx.attrs.windows_compiler_wrapper[RunInfo],
                ctx.attrs.msvc_tools[VisualStudio].cl_exe,
            ),
            os = ScriptOs("windows"),
        )
    else:
        return cmd_args(ctx.attrs.compiler)

system_cxx_toolchain = rule(
    impl = _system_cxx_toolchain_impl,
    attrs = {
        "c_flags": attrs.list(attrs.string(), default = []),
        "compiler": attrs.string(default = "cl.exe" if host_info().os.is_windows else "clang"),
        "compiler_type": attrs.string(default = "windows" if host_info().os.is_windows else "clang"),  # one of CxxToolProviderType
        "cpp_dep_tracking_mode": attrs.string(default = "makefile"),
        "cxx_compiler": attrs.string(default = "cl.exe" if host_info().os.is_windows else "clang++"),
        "cxx_flags": attrs.list(attrs.string(), default = []),
        "link_flags": attrs.list(attrs.string(), default = []),
        "link_style": attrs.string(default = "shared"),
        "linker": attrs.string(default = "link.exe" if host_info().os.is_windows else "clang++"),
        "linker_wrapper": attrs.default_only(attrs.dep(providers = [RunInfo], default = "prelude//cxx/tools:linker_wrapper")),
        "make_comp_db": attrs.default_only(attrs.dep(providers = [RunInfo], default = "prelude//cxx/tools:make_comp_db")),
        "msvc_tools": attrs.default_only(attrs.dep(providers = [VisualStudio], default = "toolchains//msvc:msvc_tools")),
        "windows_compiler_wrapper": attrs.default_only(attrs.dep(providers = [RunInfo], default = "prelude//cxx/tools:windows_compiler_wrapper")),
    },
    is_toolchain_rule = True,
)

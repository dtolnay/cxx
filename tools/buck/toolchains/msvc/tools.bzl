load("@prelude//utils:cmd_script.bzl", "ScriptOs", "cmd_script")

VisualStudio = provider(fields = [
    # Path to cl.exe
    "cl_exe",
    # Path to lib.exe
    "lib_exe",
    # Path to ml64.exe
    "ml64_exe",
])

def _find_msvc_tools_impl(ctx: AnalysisContext) -> ["provider"]:
    cl_exe_json = ctx.actions.declare_output("cl.exe.json")
    lib_exe_json = ctx.actions.declare_output("lib.exe.json")
    ml64_exe_json = ctx.actions.declare_output("ml64.exe.json")

    cmd = [
        ctx.attrs.vswhere[RunInfo],
        cmd_args("--cl=", cl_exe_json.as_output(), delimiter = ""),
        cmd_args("--lib=", lib_exe_json.as_output(), delimiter = ""),
        cmd_args("--ml64=", ml64_exe_json.as_output(), delimiter = ""),
    ]

    ctx.actions.run(
        cmd,
        category = "vswhere",
        local_only = True,
    )

    run_msvc_tool = ctx.attrs.run_msvc_tool[RunInfo]
    cl_exe_script = cmd_script(
        ctx = ctx,
        name = "cl",
        cmd = cmd_args(run_msvc_tool, cl_exe_json),
        os = ScriptOs("windows"),
    )
    lib_exe_script = cmd_script(
        ctx = ctx,
        name = "lib",
        cmd = cmd_args(run_msvc_tool, lib_exe_json),
        os = ScriptOs("windows"),
    )
    ml64_exe_script = cmd_script(
        ctx = ctx,
        name = "ml64",
        cmd = cmd_args(run_msvc_tool, ml64_exe_json),
        os = ScriptOs("windows"),
    )

    return [
        DefaultInfo(),
        VisualStudio(
            cl_exe = cl_exe_script,
            lib_exe = lib_exe_script,
            ml64_exe = ml64_exe_script,
        ),
    ]

find_msvc_tools = rule(
    impl = _find_msvc_tools_impl,
    attrs = {
        "run_msvc_tool": attrs.default_only(attrs.dep(providers = [RunInfo], default = "toolchains//msvc:run_msvc_tool")),
        "vswhere": attrs.default_only(attrs.dep(providers = [RunInfo], default = "toolchains//msvc:vswhere")),
    },
)

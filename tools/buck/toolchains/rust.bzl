load("@prelude//rust:rust_toolchain.bzl", "PanicRuntime", "RustToolchainInfo")

def _rust_toolchain_impl(ctx):
    return [
        DefaultInfo(),
        RustToolchainInfo(
            advanced_unstable_linking = True,
            clippy_driver = RunInfo(args = ["clippy-driver"]),
            compiler = RunInfo(args = ["rustc"]),
            panic_runtime = PanicRuntime("unwind"),
            rustdoc = RunInfo(args = ["rustdoc"]),
        ),
    ]

rust_toolchain = rule(
    impl = _rust_toolchain_impl,
    attrs = {},
    is_toolchain_rule = True,
)

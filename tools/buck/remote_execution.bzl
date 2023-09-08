load("@prelude//:build_mode.bzl", "BuildModeInfo")

def _remote_execution_action_key_providers_impl(ctx: AnalysisContext) -> list[Provider]:
    return [
        DefaultInfo(),
        BuildModeInfo(),
    ]

remote_execution_action_key_providers = rule(
    impl = _remote_execution_action_key_providers_impl,
    attrs = {},
)

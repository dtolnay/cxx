load("//third-party/bazel:crates.bzl", _crate_repositories = "crate_repositories")

def _crate_repositories_impl(module_ctx):
    _crate_repositories()
    return module_ctx.extension_metadata(
        root_module_direct_deps = ["vendor"],
        root_module_direct_dev_deps = [],
    )

crate_repositories = module_extension(
    implementation = _crate_repositories_impl,
)

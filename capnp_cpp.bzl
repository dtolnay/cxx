load("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")

URL = "https://github.com/capnproto/capnproto/tarball/982233268af236543df3250bac114b04bcfdc63e"
STRIP_PREFIX = "capnproto-capnproto-9822332/c++"
SHA256 = "19f33e14e41cba44fb12eb3f883cf39b1dd5c84c94165c587b9271ceb66c8d47"
TYPE = "tgz"
COMMIT = "982233268af236543df3250bac114b04bcfdc63e"

def _capnp_cpp(ctx):
    http_archive(
        name = "capnp-cpp",
        url = URL,
        strip_prefix = STRIP_PREFIX,
        type = TYPE,
        sha256 = SHA256,
    )

capnp_cpp = module_extension(implementation = _capnp_cpp)

load("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")


URL = "https://github.com/capnproto/capnproto/tarball/32e286fa989ce7dd5d3b894f3b3dbf3410c2cc80"
STRIP_PREFIX = "capnproto-capnproto-32e286f/c++"
SHA256 = "53dce39de42477891ef1208134820242078a0a71d88ccb4ff108cb632726bdc4"
TYPE = "tgz"
COMMIT = "32e286fa989ce7dd5d3b894f3b3dbf3410c2cc80"

def _capnp_cpp(ctx):
    http_archive(
        name = "capnp-cpp",
        url = URL,
        strip_prefix = STRIP_PREFIX,
        type = TYPE,
        sha256 = SHA256,
    )

capnp_cpp = module_extension(implementation = _capnp_cpp)

load("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")

URL = "https://github.com/capnproto/capnproto/tarball/fd6aad7ca96cf4c9e2bcbd74d4132691bfa8e898"
STRIP_PREFIX = "capnproto-capnproto-fd6aad7/c++"
SHA256 = "b2c065b37cd6daac4d30943bd1574f6e70b59ac1a217c859cef7d0cf7ba94efa"
TYPE = "tgz"
COMMIT = "fd6aad7ca96cf4c9e2bcbd74d4132691bfa8e898"

def _capnp_cpp(ctx):
    http_archive(
        name = "capnp-cpp",
        url = URL,
        strip_prefix = STRIP_PREFIX,
        type = TYPE,
        sha256 = SHA256,
    )

capnp_cpp = module_extension(implementation = _capnp_cpp)

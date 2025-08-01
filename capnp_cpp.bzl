load("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")


URL = "https://github.com/capnproto/capnproto/tarball/5cbfc8e03033c1066d0e1309768fdb5f5fe2a6e4"
STRIP_PREFIX = "capnproto-capnproto-5cbfc8e/c++"
SHA256 = "808f586689445f5237358a52f4f9afbed953d8263982ad2c8db23597ba039c5d"
TYPE = "tgz"
COMMIT = "5cbfc8e03033c1066d0e1309768fdb5f5fe2a6e4"

def _capnp_cpp(ctx):
    http_archive(
        name = "capnp-cpp",
        url = URL,
        strip_prefix = STRIP_PREFIX,
        type = TYPE,
        sha256 = SHA256,
    )

capnp_cpp = module_extension(implementation = _capnp_cpp)

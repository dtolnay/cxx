load("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")


URL = "https://github.com/capnproto/capnproto/tarball/f53fcccb97c3688b611fc55eb214665a3a999893"
STRIP_PREFIX = "capnproto-capnproto-f53fccc/c++"
SHA256 = "ba3ba8503cffb12b53daf2420c48565dd342492e88d64573d6368edbe33fd79a"
TYPE = "tgz"
COMMIT = "f53fcccb97c3688b611fc55eb214665a3a999893"

def _capnp_cpp(ctx):
    http_archive(
        name = "capnp-cpp",
        url = URL,
        strip_prefix = STRIP_PREFIX,
        type = TYPE,
        sha256 = SHA256,
    )

capnp_cpp = module_extension(implementation = _capnp_cpp)

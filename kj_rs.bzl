load("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")


URL = "https://github.com/capnproto/kj-rs/tarball/df8a25bb2c4d3c4e361d84011a6eac640b278b92"
STRIP_PREFIX = "capnproto-kj-rs-df8a25b"
SHA256 = "d38626cbae30443fef4d009f18c5dad1af3da342f8275a6e833c1691bbb090cc"
TYPE = "tgz"
COMMIT = "df8a25bb2c4d3c4e361d84011a6eac640b278b92"

def _kj_rs(ctx):
    http_archive(
        name = "kj-rs",
        url = URL,
        strip_prefix = STRIP_PREFIX,
        type = TYPE,
        sha256 = SHA256,
    )

kj_rs = module_extension(implementation = _kj_rs)

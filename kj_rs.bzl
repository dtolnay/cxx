load("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")


URL = "https://github.com/capnproto/kj-rs/tarball/82f9a0864725cfd9676ef270664c8e2c81ecd1b9"
STRIP_PREFIX = "capnproto-kj-rs-82f9a08"
SHA256 = "2e2a871733c7e0de995769413efcf62c8c4580bf671a27d5f33832e12c60fffa"
TYPE = "tgz"
COMMIT = "82f9a0864725cfd9676ef270664c8e2c81ecd1b9"

def _kj_rs(_ctx):
    http_archive(
        name = "kj-rs",
        url = URL,
        strip_prefix = STRIP_PREFIX,
        type = TYPE,
        sha256 = SHA256,
    )

kj_rs = module_extension(implementation = _kj_rs)

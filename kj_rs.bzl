load("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")


URL = "https://github.com/capnproto/kj-rs/tarball/13f2152e0cca109b0c64f9eeca4f62bcf0689a87"
STRIP_PREFIX = "capnproto-kj-rs-13f2152"
SHA256 = "bcd3b2b22a422936bebb5fcefa56eb7d4fa076e329fb6214683c4d6b5e7ec4c3"
TYPE = "tgz"
COMMIT = "13f2152e0cca109b0c64f9eeca4f62bcf0689a87"

def _kj_rs(_ctx):
    http_archive(
        name = "kj-rs",
        url = URL,
        strip_prefix = STRIP_PREFIX,
        type = TYPE,
        sha256 = SHA256,
    )

kj_rs = module_extension(implementation = _kj_rs)

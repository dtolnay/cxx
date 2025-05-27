"List of cargo crate dependencies"
load("@rules_rust//crate_universe:defs.bzl", "crate")

PACKAGES = {
    "cc": crate.spec(version = "1"),
    "clap": crate.spec(default_features = False, features = ["derive", "std", "help"], version = "4"),
    "codespan-reporting": crate.spec(version = "0"),
    "foldhash": crate.spec(version = "0"),
    "proc-macro2": crate.spec(features = ["span-locations"], version = "1"),
    "quote": crate.spec(version = "1"),
    "rustversion": crate.spec(version = "1"),
    "scratch": crate.spec(version = "1"),
    "static_assertions": crate.spec(version = "1"),
    "syn": crate.spec(features = ["full"], version = "2"),
}

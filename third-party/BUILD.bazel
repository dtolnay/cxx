load("@rules_rust//crate_universe:defs.bzl", "crates_vendor")

crates_vendor(
    name = "vendor",
    cargo_lockfile = "//third-party:Cargo.lock",
    generate_build_scripts = True,
    manifests = ["//third-party:Cargo.toml"],
    mode = "remote",
    tags = ["manual"],
    vendor_path = "bazel",
)

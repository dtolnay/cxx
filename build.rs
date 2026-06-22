use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    let manifest_dir_opt = env::var_os("CARGO_MANIFEST_DIR").map(PathBuf::from);

    let out_dir = PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR shall be set by cargo"));
    cxx_cc::build_cxxcc_if_cc_stage(&PathBuf::from(&out_dir)).expect("failed to compile cxx.cc");

    println!("cargo:rerun-if-changed=include/cxx.h");
    println!("cargo:rustc-cfg=built_with_cargo");

    if let Some(manifest_dir) = &manifest_dir_opt {
        let cxx_h = manifest_dir.join("include/cxx.h");
        println!("cargo:HEADER={}", cxx_h.to_string_lossy());
    }

    println!("cargo:rustc-check-cfg=cfg(built_with_cargo)");
    println!("cargo:rustc-check-cfg=cfg(compile_error_if_alloc)");
    println!("cargo:rustc-check-cfg=cfg(compile_error_if_std)");
    println!("cargo:rustc-check-cfg=cfg(cxx_experimental_no_alloc)");
    println!("cargo:rustc-check-cfg=cfg(skip_ui_tests)");

    if let Some(rustc) = rustc_version() {
        if rustc.minor < 85 {
            println!("cargo:warning=The cxx crate requires a rustc version 1.85.0 or newer.");
            println!(
                "cargo:warning=You appear to be building with: {}",
                rustc.version,
            );
        }
    }

    if let (Some(manifest_links), Some(pkg_version_major)) = (
        env::var_os("CARGO_MANIFEST_LINKS"),
        env::var_os("CARGO_PKG_VERSION_MAJOR"),
    ) {
        assert_eq!(
            manifest_links,
            *format!("cxxbridge{}", pkg_version_major.to_str().unwrap()),
        );
    }
}

struct RustVersion {
    version: String,
    minor: u32,
}

fn rustc_version() -> Option<RustVersion> {
    let rustc = env::var_os("RUSTC")?;
    let output = Command::new(rustc).arg("--version").output().ok()?;
    let version = String::from_utf8(output.stdout).ok()?;
    let mut pieces = version.split('.');
    if pieces.next() != Some("rustc 1") {
        return None;
    }
    let minor = pieces.next()?.parse().ok()?;
    Some(RustVersion { version, minor })
}

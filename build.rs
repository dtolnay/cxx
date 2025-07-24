#![allow(unknown_lints)]
#![allow(unexpected_cfgs)]

use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    let manifest_dir_opt = env::var_os("CARGO_MANIFEST_DIR").map(PathBuf::from);
    let manifest_dir = manifest_dir_opt.as_deref().unwrap_or(Path::new(""));

    cc::Build::new()
        .file(manifest_dir.join("src/cxx.cc"))
        .cpp(true)
        .cpp_link_stdlib(None) // linked via link-cplusplus crate
        .std(cxxbridge_flags::STD)
        .warnings_into_errors(cfg!(deny_warnings))
        .compile("cxxbridge1");

    println!("cargo:rerun-if-changed=src/cxx.cc");
    println!("cargo:rerun-if-changed=include/cxx.h");
    println!("cargo:rustc-cfg=built_with_cargo");

    if let Some(manifest_dir) = &manifest_dir_opt {
        let cxx_h = manifest_dir.join("include").join("cxx.h");
        println!("cargo:HEADER={}", cxx_h.to_string_lossy());
    }

    if let Some(rustc) = rustc_version() {
        if rustc.minor >= 80 {
            println!("cargo:rustc-check-cfg=cfg(built_with_cargo)");
            println!("cargo:rustc-check-cfg=cfg(compile_error_if_alloc)");
            println!("cargo:rustc-check-cfg=cfg(compile_error_if_std)");
            println!("cargo:rustc-check-cfg=cfg(cxx_experimental_no_alloc)");
            println!("cargo:rustc-check-cfg=cfg(error_in_core)");
            println!("cargo:rustc-check-cfg=cfg(seek_relative)");
            println!("cargo:rustc-check-cfg=cfg(skip_ui_tests)");
        }

        if rustc.minor < 73 {
            println!("cargo:warning=The cxx crate requires a rustc version 1.73.0 or newer.");
            println!(
                "cargo:warning=You appear to be building with: {}",
                rustc.version,
            );
        }

        if rustc.minor >= 80 {
            // std::io::Seek::seek_relative
            println!("cargo:rustc-cfg=seek_relative");
        }

        if rustc.minor >= 81 {
            // core::error::Error
            println!("cargo:rustc-cfg=error_in_core");
        }
    }

    persist_target_triple();
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

/// `tests/cpp_ui_tests.rs` needs to know the target triple when invoking a
/// C/C++ compiler through the `cc` crate.  The function below facilitates this
/// by capturing the value of the `TARGET` environment variable seen during
/// `build.rs` execution, and writing this value to a file that the
/// `cpp_ui_tests` can pick up using `include_str!`.
///
/// An alternative approach would be to drive `cpp_ui_tests` from `build.rs`
/// during build time.  This seems less desirable than the current approach,
/// which benefits from being a set of regular test cases (which can be
/// filtered, have their stderr captured, etc.).  FWIW the `tests/ui` tests also
/// invoke build tools (e.g. `rustc`) at test time, rather than build time, so
/// this seems okay.
///
/// This function ignores errors, because we don't want to avoid disrupting
/// production builds (even if failure to generate `target_triple.txt` may
/// disrupt test builds).
fn persist_target_triple() {
    let Some(out_dir) = env::var_os("OUT_DIR") else {
        return;
    };
    let Ok(target) = env::var("TARGET") else {
        return;
    };
    println!("cargo:rerun-if-env-changed=TARGET");

    let out_dir = Path::new(&out_dir);
    let _ = std::fs::write(out_dir.join("target_triple.txt"), target);
}

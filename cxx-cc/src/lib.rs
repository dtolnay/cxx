#![expect(unexpected_cfgs)]

use std::io::Result;
use std::path::Path;

const CXX_H: &[u8] = include_bytes!("../../include/cxx.h");
const CXX_CC: &[u8] = include_bytes!("cxx.cc");

fn build_cxxcc(dir: &Path) -> Result<()> {
    std::fs::write(dir.join("cxx.h"), CXX_H)?;
    let cxx_cc_path = dir.join("cxx.cc");
    std::fs::write(&cxx_cc_path, CXX_CC)?;
    cc::Build::new()
        .file(&cxx_cc_path)
        .cpp(true)
        .cpp_link_stdlib(None) // linked via link-cplusplus crate
        .std(cxxbridge_flags::STD)
        .warnings_into_errors(cfg!(deny_warnings))
        .compile("cxxbridge1");
    Ok(())
}

/// If the `compile-at-bridge-stage` feature is enabled, does nothing.
/// Otherwise, builds `cxx.cc`.
///
/// # Errors
/// Filesystem errors while preparing the compilation.
pub fn build_cxxcc_if_cc_stage(dir: &Path) -> Result<()> {
    if cfg!(feature = "compile-at-bridge-stage") {
        Ok(())
    } else {
        build_cxxcc(dir)
    }
}

/// If the `compile-at-bridge-stage` feature is enabled, builds `cxx.cc`.
/// Otherwise, does nothing.
///
/// # Errors
/// Filesystem errors while preparing the compilation.
pub fn build_cxxcc_if_bridge_stage(dir: &Path) -> Result<()> {
    if cfg!(feature = "compile-at-bridge-stage") {
        build_cxxcc(dir)
    } else {
        Ok(())
    }
}

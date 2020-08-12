use crate::error::{Error, Result};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

fn out_dir() -> Result<PathBuf> {
    env::var_os("OUT_DIR")
        .map(PathBuf::from)
        .ok_or(Error::MissingOutDir)
}

fn manifest_dir() -> Result<PathBuf> {
    std::env::var_os("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .ok_or(Error::MissingManifestDir)
}

pub(crate) fn cc_build() -> cc::Build {
    try_cc_build().unwrap_or_default()
}

fn try_cc_build() -> Result<cc::Build> {
    let mut build = cc::Build::new();
    build.include(include_dir()?);
    build.include(out_dir()?);
    build.include(manifest_dir()?);
    Ok(build)
}

fn relative_to_cargo_manifest_dir(original: &Path) -> Result<PathBuf> {
    let manifest_dir = canonicalize(manifest_dir()?)?;
    let original = canonicalize(original)?;

    original
        .strip_prefix(&manifest_dir)
        .map(|p| p.to_path_buf())
        .map_err(|_| Error::CargoManifestDirNotParent {
            manifest_dir,
            child: original,
        })
}

pub(crate) fn out_with_extension(path: &Path, ext: &str) -> Result<PathBuf> {
    let mut file_name = path.file_name().unwrap().to_owned();
    file_name.push(ext);

    let out_dir = out_dir()?;
    let rel = relative_to_cargo_manifest_dir(path)?;
    Ok(out_dir.join(rel).with_file_name(file_name))
}

pub(crate) fn include_dir() -> Result<PathBuf> {
    out_dir().map(|p| p.join("cxxbridge"))
}

#[cfg(not(windows))]
fn canonicalize(path: impl AsRef<Path>) -> Result<PathBuf> {
    Ok(fs::canonicalize(path)?)
}

#[cfg(windows)]
fn canonicalize(path: impl AsRef<Path>) -> Result<PathBuf> {
    // Real fs::canonicalize on Windows produces UNC paths which cl.exe is
    // unable to handle in includes. Use a poor approximation instead.
    // https://github.com/rust-lang/rust/issues/42869
    // https://github.com/alexcrichton/cc-rs/issues/169
    Ok(env::current_dir()?.join(path))
}

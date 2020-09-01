use crate::cargo;
use crate::error::{Error, Result};
use crate::gen::fs;
use std::env;
use std::ops::Deref;
use std::path::{Path, PathBuf};

pub(crate) struct TargetDir(pub PathBuf);

impl Deref for TargetDir {
    type Target = Path;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

fn out_dir() -> Result<PathBuf> {
    env::var_os("OUT_DIR")
        .map(PathBuf::from)
        .ok_or(Error::MissingOutDir)
}

pub(crate) fn cc_build(target_dir: &TargetDir) -> cc::Build {
    let mut build = cc::Build::new();
    build.include(include_dir(target_dir));
    build.include(target_dir.parent().unwrap());
    build
}

// Symlink the header file into a predictable place. The header generated from
// path/to/mod.rs gets linked to targets/cxxbridge/path/to/mod.rs.h.
pub(crate) fn symlink_header(path: &Path, original: &Path, target_dir: &TargetDir) {
    let _ = try_symlink_header(path, original, target_dir);
}

fn try_symlink_header(path: &Path, original: &Path, target_dir: &TargetDir) -> Result<()> {
    let suffix = relative_to_parent_of_target_dir(original, target_dir)?;
    let ref dst = include_dir(target_dir).join(suffix);

    fs::create_dir_all(dst.parent().unwrap())?;
    let _ = fs::remove_file(dst);
    symlink_or_copy(path, dst)?;

    let mut file_name = dst.file_name().unwrap().to_os_string();
    file_name.push(".h");
    let ref dst2 = dst.with_file_name(file_name);
    symlink_or_copy(path, dst2)?;

    Ok(())
}

fn relative_to_parent_of_target_dir(original: &Path, target_dir: &TargetDir) -> Result<PathBuf> {
    let mut outer = target_dir.parent().unwrap();
    let original = canonicalize(original)?;
    loop {
        if let Ok(suffix) = original.strip_prefix(outer) {
            return Ok(suffix.to_owned());
        }
        match outer.parent() {
            Some(parent) => outer = parent,
            None => return Ok(original.components().skip(1).collect()),
        }
    }
}

pub(crate) fn out_with_extension(
    path: &Path,
    ext: &str,
    target_dir: &TargetDir,
) -> Result<PathBuf> {
    let mut file_name = path.file_name().unwrap().to_owned();
    file_name.push(ext);

    let out_dir = out_dir()?;
    let rel = relative_to_parent_of_target_dir(path, target_dir)?;
    Ok(out_dir.join(rel).with_file_name(file_name))
}

pub(crate) fn include_dir(target_dir: &TargetDir) -> PathBuf {
    target_dir.join("cxxbridge")
}

pub(crate) fn target_dir() -> Result<TargetDir> {
    let fallback_err = match cargo::target_dir() {
        Ok(target_dir) => return Ok(target_dir),
        Err(err) => Error::TargetDir(err),
    };

    // Fallback if Cargo did not work.
    let mut dir = out_dir().and_then(canonicalize)?;
    loop {
        if dir.ends_with("target") {
            return Ok(TargetDir(dir));
        }
        if !dir.pop() {
            return Err(fallback_err);
        }
    }
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
    Ok(fs::current_dir()?.join(path))
}

#[cfg(unix)]
use self::fs::symlink_file as symlink_or_copy;

#[cfg(windows)]
fn symlink_or_copy(src: &Path, dst: &Path) -> Result<()> {
    // Pre-Windows 10, symlinks require admin privileges. Since Windows 10, they
    // require Developer Mode. If it fails, fall back to copying the file.
    if fs::symlink_file(src, dst).is_err() {
        fs::copy(src, dst)?;
    }
    Ok(())
}

#[cfg(not(any(unix, windows)))]
use self::fs::copy as symlink_or_copy;

use crate::error::{Error, Result};
use std::env;
use std::fs;
use std::os;
use std::path::{Path, PathBuf};

fn out_dir() -> Result<PathBuf> {
    env::var_os("OUT_DIR")
        .map(PathBuf::from)
        .ok_or(Error::MissingOutDir)
}

pub(crate) fn cc_build() -> cc::Build {
    try_cc_build().unwrap_or_default()
}

fn try_cc_build() -> Result<cc::Build> {
    let target_dir = target_dir()?;

    let mut build = cc::Build::new();
    build.include(target_dir.join("cxxbridge"));
    build.include(target_dir.parent().unwrap());
    Ok(build)
}

// Symlink the header file into a predictable place. The header generated from
// path/to/mod.rs gets linked to targets/cxxbridge/path/to/mod.h.
pub(crate) fn symlink_header(path: &Path, original: &Path) {
    let _ = try_symlink_header(path, original);
}

fn try_symlink_header(path: &Path, original: &Path) -> Result<()> {
    let suffix = relative_to_parent_of_target_dir(original)?;
    let ref dst = target_dir()?.join("cxxbridge").join(suffix);

    fs::create_dir_all(dst.parent().unwrap())?;
    let _ = fs::remove_file(dst);
    #[cfg(unix)]
    os::unix::fs::symlink(path, dst)?;
    #[cfg(windows)]
    os::windows::fs::symlink_file(path, dst)?;

    Ok(())
}

fn relative_to_parent_of_target_dir(original: &Path) -> Result<PathBuf> {
    let target_dir = target_dir()?;
    let mut outer = target_dir.parent().unwrap();
    let original = original.canonicalize()?;
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

pub(crate) fn out_with_extension(path: &Path, ext: &str) -> Result<PathBuf> {
    let mut file_name = path.file_name().unwrap().to_owned();
    file_name.push(ext);

    let out_dir = out_dir()?;
    let rel = relative_to_parent_of_target_dir(path)?;
    Ok(out_dir.join(rel).with_file_name(file_name))
}

fn target_dir() -> Result<PathBuf> {
    let mut dir = out_dir()?.canonicalize()?;
    loop {
        if dir.ends_with("target") {
            return Ok(dir);
        }
        if !dir.pop() {
            return Err(Error::TargetDir);
        }
    }
}

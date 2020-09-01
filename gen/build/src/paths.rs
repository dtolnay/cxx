use crate::error::{Error, Result};
use crate::gen::fs;
use crate::Project;
use std::env;
use std::path::{Path, PathBuf};

pub(crate) enum TargetDir {
    Path(PathBuf),
    Unknown,
}

pub(crate) fn out_dir() -> Result<PathBuf> {
    env::var_os("OUT_DIR")
        .map(PathBuf::from)
        .ok_or(Error::MissingOutDir)
}

pub(crate) fn cc_build(prj: &Project) -> cc::Build {
    let mut build = cc::Build::new();
    build.include(include_dir(prj));
    if let TargetDir::Path(target_dir) = &prj.target_dir {
        build.include(target_dir.parent().unwrap());
    }
    build
}

// Symlink the header file into a predictable place. The header generated from
// path/to/mod.rs gets linked to target/cxxbridge/path/to/mod.rs.h.
pub(crate) fn symlink_header(prj: &Project, path: &Path, original: &Path) {
    if let TargetDir::Unknown = prj.target_dir {
        return;
    }
    let _ = try_symlink_header(prj, path, original);
}

fn try_symlink_header(prj: &Project, path: &Path, original: &Path) -> Result<()> {
    let suffix = relative_to_parent_of_target_dir(prj, original)?;
    let ref dst = include_dir(prj).join(suffix);

    fs::create_dir_all(dst.parent().unwrap())?;
    let _ = fs::remove_file(dst);
    symlink_or_copy(path, dst)?;

    let mut file_name = dst.file_name().unwrap().to_os_string();
    file_name.push(".h");
    let ref dst2 = dst.with_file_name(file_name);
    symlink_or_copy(path, dst2)?;

    Ok(())
}

fn relative_to_parent_of_target_dir(prj: &Project, original: &Path) -> Result<PathBuf> {
    let mut outer = match &prj.target_dir {
        TargetDir::Path(target_dir) => target_dir.parent().unwrap(),
        TargetDir::Unknown => unimplemented!(), // FIXME
    };
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

pub(crate) fn out_with_extension(prj: &Project, path: &Path, ext: &str) -> Result<PathBuf> {
    let mut file_name = path.file_name().unwrap().to_owned();
    file_name.push(ext);

    let rel = relative_to_parent_of_target_dir(prj, path)?;
    Ok(prj.out_dir.join(rel).with_file_name(file_name))
}

pub(crate) fn include_dir(prj: &Project) -> PathBuf {
    match &prj.target_dir {
        TargetDir::Path(target_dir) => target_dir.join("cxxbridge"),
        TargetDir::Unknown => prj.out_dir.join("cxxbridge"),
    }
}

pub(crate) fn search_parents_for_target_dir(out_dir: &Path) -> TargetDir {
    let mut dir = match out_dir.canonicalize() {
        Ok(dir) => dir,
        Err(_) => return TargetDir::Unknown,
    };
    loop {
        let is_target = dir.ends_with("target");
        let parent_contains_cargo_toml = dir.with_file_name("Cargo.toml").exists();
        if is_target && parent_contains_cargo_toml {
            return TargetDir::Path(dir);
        }
        if !dir.pop() {
            return TargetDir::Unknown;
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

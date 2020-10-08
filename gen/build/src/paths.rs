use crate::error::{Error, Result};
use crate::gen::fs;
use crate::Project;
use std::env;
use std::ffi::{OsStr, OsString};
use std::path::{Component, Path, PathBuf};

pub(crate) enum TargetDir {
    Path(PathBuf),
    Unknown,
}

pub(crate) fn out_dir() -> Result<PathBuf> {
    env::var_os("OUT_DIR")
        .map(PathBuf::from)
        .ok_or(Error::MissingOutDir)
}

// Given a path provided by the user, determines where generated files related
// to that path should go in our out dir. In particular we don't want to
// accidentally write generated code upward of our out dir, even if the user
// passed a path containing lots of `..` or an absolute path.
pub(crate) fn local_relative_path(path: &Path) -> PathBuf {
    let mut rel_path = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Prefix(_) | Component::RootDir | Component::CurDir => {}
            Component::ParentDir => drop(rel_path.pop()), // noop if empty
            Component::Normal(name) => rel_path.push(name),
        }
    }
    rel_path
}

pub(crate) fn namespaced(base: &Path, rel_path: &Path) -> PathBuf {
    let mut path = base.to_owned();
    path.push("cxxbridge");
    path.extend(package_name());
    path.push(rel_path);
    path
}

pub(crate) trait PathExt {
    fn with_appended_extension(&self, suffix: impl AsRef<OsStr>) -> PathBuf;
}

impl PathExt for Path {
    fn with_appended_extension(&self, suffix: impl AsRef<OsStr>) -> PathBuf {
        let mut file_name = self.file_name().unwrap().to_owned();
        file_name.push(suffix);
        self.with_file_name(file_name)
    }
}

pub(crate) fn include_dir(prj: &Project) -> PathBuf {
    match &prj.target_dir {
        TargetDir::Path(target_dir) => target_dir.join("cxxbridge"),
        TargetDir::Unknown => prj.out_dir.join("cxxbridge"),
    }
}

pub(crate) fn manifest_dir() -> Option<PathBuf> {
    env::var_os("CARGO_MANIFEST_DIR").map(PathBuf::from)
}

pub(crate) fn package_name() -> Option<OsString> {
    env::var_os("CARGO_PKG_NAME")
}

pub(crate) fn search_parents_for_target_dir(out_dir: &Path) -> TargetDir {
    // fs::canonicalize on Windows produces UNC paths which cl.exe is unable to
    // handle in includes.
    // https://github.com/rust-lang/rust/issues/42869
    // https://github.com/alexcrichton/cc-rs/issues/169
    let mut also_try_canonical = cfg!(not(windows));

    let mut dir = out_dir.to_owned();
    loop {
        let is_target = dir.ends_with("target");
        let parent_contains_cargo_toml = dir.with_file_name("Cargo.toml").exists();
        if is_target && parent_contains_cargo_toml {
            return TargetDir::Path(dir);
        }
        if dir.pop() {
            continue;
        }
        if also_try_canonical {
            if let Ok(canonical_dir) = out_dir.canonicalize() {
                dir = canonical_dir;
                also_try_canonical = false;
                continue;
            }
        }
        return TargetDir::Unknown;
    }
}

#[cfg(unix)]
pub(crate) use self::fs::symlink_file as symlink_or_copy;

#[cfg(windows)]
pub(crate) fn symlink_or_copy(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> fs::Result<()> {
    // Pre-Windows 10, symlinks require admin privileges. Since Windows 10, they
    // require Developer Mode. If it fails, fall back to copying the file.
    let src = src.as_ref();
    let dst = dst.as_ref();
    if fs::symlink_file(src, dst).is_err() {
        fs::copy(src, dst)?;
    }
    Ok(())
}

#[cfg(not(any(unix, windows)))]
pub(crate) use self::fs::copy as symlink_or_copy;

#[cfg(any(unix, windows))]
pub(crate) use self::fs::symlink_dir;

#[cfg(not(any(unix, windows)))]
pub(crate) fn symlink_dir(_src: impl AsRef<Path>, _dst: impl AsRef<Path>) -> fs::Result<()> {
    Ok(())
}

#[cfg(not(windows))]
pub(crate) use self::fs::remove_file as remove_symlink_dir;

// On Windows, trying to use remove_file to remove a symlink which points to a
// directory fails with "Access is denied".
#[cfg(windows)]
pub(crate) use self::fs::remove_dir as remove_symlink_dir;

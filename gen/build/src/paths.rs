use crate::error::Result;
use crate::gen::fs;
use std::ffi::OsStr;
use std::path::{Component, Path, PathBuf};

pub(crate) fn manifest_dir() -> Result<PathBuf> {
    crate::env_os("CARGO_MANIFEST_DIR").map(PathBuf::from)
}

pub(crate) fn out_dir() -> Result<PathBuf> {
    crate::env_os("OUT_DIR").map(PathBuf::from)
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

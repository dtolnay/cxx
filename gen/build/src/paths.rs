use crate::error::{Error, Result};
use crate::gen::fs;
use crate::Project;
use std::env;
use std::ffi::OsString;
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
    let mut dst = include_dir(prj);
    dst.extend(package_name());
    dst.push(original);

    let parent = dst.parent().unwrap();
    fs::create_dir_all(parent)?;
    let _ = fs::remove_file(&dst);
    symlink_or_copy(path, &dst)?;

    let mut file_name = dst.file_name().unwrap().to_os_string();
    file_name.push(".h");
    let ref dst2 = dst.with_file_name(file_name);
    symlink_or_copy(path, dst2)?;

    Ok(())
}

pub(crate) fn out_with_extension(prj: &Project, rel_path: &Path, ext: &str) -> PathBuf {
    let mut file_name = rel_path.file_name().unwrap().to_owned();
    file_name.push(ext);

    let mut res = prj.out_dir.clone();
    res.push("cxxbridge");
    res.extend(package_name());
    res.push(rel_path);
    res.with_file_name(file_name)
}

pub(crate) fn include_dir(prj: &Project) -> PathBuf {
    match &prj.target_dir {
        TargetDir::Path(target_dir) => target_dir.join("cxxbridge"),
        TargetDir::Unknown => prj.out_dir.join("cxxbridge"),
    }
}

fn package_name() -> Option<OsString> {
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

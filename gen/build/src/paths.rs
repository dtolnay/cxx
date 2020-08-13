use crate::error::{Error, Result};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

pub(crate) enum RelativeToDir {
    Workspace,
    Manifest,
}

impl RelativeToDir {
    fn dir(self) -> Result<PathBuf> {
        match self {
            Self::Workspace => workspace_dir(),
            Self::Manifest => manifest_dir(),
        }
    }
}

pub(crate) fn out_dir() -> Result<PathBuf> {
    env::var_os("OUT_DIR")
        .map(PathBuf::from)
        .ok_or(Error::MissingOutDir)
}

fn manifest_dir() -> Result<PathBuf> {
    std::env::var_os("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .ok_or(Error::MissingManifestDir)
}

fn workspace_dir() -> Result<PathBuf> {
    let manifest_dir = manifest_dir()?;
    let mut workspace_dir = manifest_dir.clone();
    while workspace_dir.pop() {
        let cargo = workspace_dir.join("Cargo.toml");
        if cargo.exists() {
            if let Ok(workspace) = fs::read_to_string(cargo) {
                let workspace = workspace.to_lowercase();
                if workspace.contains("[workspace]")
                    && workspace.contains(&format!(
                        r#""{}""#,
                        manifest_dir
                            .strip_prefix(&workspace_dir)
                            .unwrap()
                            .to_string_lossy()
                            .replace("\\", "/")
                    ))
                {
                    println!("Workspace dir: {}", workspace_dir.display());
                    return Ok(workspace_dir);
                }
            }
        }
    }
    println!("Not in workspace: {}", workspace_dir.display());
    return Ok(manifest_dir);
}

pub(crate) fn cc_build() -> cc::Build {
    try_cc_build().unwrap_or_default()
}

fn try_cc_build() -> Result<cc::Build> {
    let mut build = cc::Build::new();
    build.include(include_dir()?);
    build.include(out_dir()?);
    build.include(manifest_dir()?);
    build.include(workspace_dir()?);
    Ok(build)
}

fn relative_to_dir(original: &Path, relative_dir: RelativeToDir) -> Result<PathBuf> {
    let relative_dir = relative_dir.dir()?;
    let dir = canonicalize(relative_dir)?;
    let original = canonicalize(original)?;

    original
        .strip_prefix(&dir)
        .map(|p| p.to_path_buf())
        .map_err(|_| Error::CargoDirNotParent {
            manifest_dir: dir,
            child: original,
        })
}

pub(crate) fn out_with_extension(path: &Path, ext: &str, dir: RelativeToDir) -> Result<PathBuf> {
    let mut file_name = path.file_name().unwrap().to_owned();
    file_name.push(ext);

    let out_dir = out_dir()?;
    let rel = relative_to_dir(path, dir)?;
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

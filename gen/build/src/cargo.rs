use crate::paths::TargetDir;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::str;

pub(crate) fn target_dir(out_dir: &Path) -> TargetDir {
    try_target_dir(out_dir).map_or(TargetDir::Unknown, TargetDir::Path)
}

fn try_target_dir(out_dir: &Path) -> Option<PathBuf> {
    let cargo = option_env!("CARGO").unwrap_or("cargo");
    let output = Command::new(cargo)
        .current_dir(out_dir)
        .arg("metadata")
        .arg("--no-deps")
        .arg("--format-version=1")
        .output()
        .ok()?;

    // Cargo only outputs utf8 encoded JSON.
    let mut metadata = str::from_utf8(&output.stdout).ok()?;

    let key_pattern = "\"target_directory\":";
    let key_index = metadata.rfind(key_pattern)?;
    metadata = &metadata[key_index + key_pattern.len()..];
    let open_quote_index = metadata.find('"')?;
    metadata = &metadata[open_quote_index + 1..];
    let close_quote_index = metadata.find('"')?;
    let string = &metadata[..close_quote_index];
    let target_directory = string.replace("\\\\", "\\");
    Some(PathBuf::from(target_directory))
}

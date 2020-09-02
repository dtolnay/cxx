use crate::error::{Error, Result};
use crate::gen::fs;
use std::path::Path;

pub(crate) fn write(path: impl AsRef<Path>, content: &[u8]) -> Result<()> {
    let path = path.as_ref();

    let mut create_dir_error = None;
    if path.exists() {
        if let Ok(existing) = fs::read(path) {
            if existing == content {
                // Avoid bumping modified time with unchanged contents.
                return Ok(());
            }
        }
        let _ = fs::remove_file(path);
    } else {
        let parent = path.parent().unwrap();
        create_dir_error = fs::create_dir_all(parent).err();
    }

    match fs::write(path, content) {
        // As long as write succeeded, ignore any create_dir_all error.
        Ok(()) => Ok(()),
        // If create_dir_all and write both failed, prefer the first error.
        Err(err) => Err(Error::Fs(create_dir_error.unwrap_or(err))),
    }
}

use std::io;
use std::path::StripPrefixError;
use thiserror::Error;

pub(super) type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Error, Debug)]
#[error(transparent)]
pub(super) enum Error {
    #[error("missing OUT_DIR environment variable")]
    MissingOutDir,
    #[error("failed to locate target dir")]
    TargetDir,
    Io(#[from] io::Error),
    StripPrefix(#[from] StripPrefixError),
}

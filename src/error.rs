use std::io;
use thiserror::Error;

pub(super) type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Error, Debug)]
pub(super) enum Error {
    #[error("missing OUT_DIR environment variable")]
    MissingOutDir,
    #[error("failed to locate target dir")]
    TargetDir,
    #[error(transparent)]
    Io(#[from] io::Error),
}

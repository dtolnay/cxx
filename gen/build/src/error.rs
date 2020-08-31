use crate::gen::fs;
use std::error::Error as StdError;
use std::fmt::{self, Display};

pub(super) type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug)]
pub(super) enum Error {
    MissingOutDir,
    TargetDir,
    Fs(fs::Error),
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::MissingOutDir => write!(f, "missing OUT_DIR environment variable"),
            Error::TargetDir => write!(f, "failed to locate target dir"),
            Error::Fs(err) => err.fmt(f),
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Error::Fs(err) => err.source(),
            _ => None,
        }
    }
}

impl From<fs::Error> for Error {
    fn from(err: fs::Error) -> Self {
        Error::Fs(err)
    }
}

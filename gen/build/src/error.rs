use crate::gen::fs;
use std::error::Error as StdError;
use std::fmt::{self, Display};
use std::io;

pub(super) type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug)]
pub(super) enum Error {
    MissingOutDir,
    TargetDir(TargetDirError),
    Fs(fs::Error),
}

#[derive(Debug)]
pub(crate) enum TargetDirError {
    Io(io::Error),
    NotFound,
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::MissingOutDir => write!(f, "missing OUT_DIR environment variable"),
            Error::TargetDir(_) => write!(f, "unable to identify target dir"),
            Error::Fs(err) => err.fmt(f),
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Error::TargetDir(err) => match err {
                TargetDirError::Io(err) => Some(err),
                TargetDirError::NotFound => None,
            },
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

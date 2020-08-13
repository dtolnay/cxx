use std::error::Error as StdError;
use std::fmt::{self, Display};
use std::io;
use std::path::PathBuf;

pub(super) type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug)]
pub(super) enum Error {
    MissingOutDir,
    MissingManifestDir,
    CargoDirNotParent {
        manifest_dir: PathBuf,
        child: PathBuf,
    },
    Io(io::Error),
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::MissingOutDir => write!(f, "missing OUT_DIR environment variable"),
            Error::MissingManifestDir => {
                write!(f, "missing CARGO_MANIFEST_DIR environment variable")
            }
            Error::CargoDirNotParent {
                manifest_dir,
                child,
            } => write!(
                f,
                "{} is not child of {}",
                child.display(),
                manifest_dir.display()
            ),
            Error::Io(err) => err.fmt(f),
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Error::Io(err) => Some(err),
            _ => None,
        }
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::Io(err)
    }
}

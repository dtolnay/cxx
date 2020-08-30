// We can expose more detail on the error as the need arises, but start with an
// opaque error type for now.

use std::error::Error as StdError;
use std::fmt::{self, Debug, Display};

pub struct Error(pub(crate) crate::gen::Error);

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Debug::fmt(&self.0, f)
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        self.0.source()
    }
}

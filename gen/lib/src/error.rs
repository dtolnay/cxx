// We can expose more detail on the error as the need arises, but start with an
// opaque error type for now.

use std::error::Error as StdError;
use std::fmt::{self, Debug, Display};
use std::iter;

#[allow(missing_docs)]
pub struct Error {
    pub(crate) err: crate::gen::Error,
}

impl Error {
    /// Returns the span of the error, if available.
    pub fn span(&self) -> Option<proc_macro2::Span> {
        match &self.err {
            crate::gen::Error::Syn(err) => Some(err.span()),
            _ => None,
        }
    }
}

impl From<crate::gen::Error> for Error {
    fn from(err: crate::gen::Error) -> Self {
        Error { err }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(&self.err, f)
    }
}

impl Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Debug::fmt(&self.err, f)
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        self.err.source()
    }
}

impl IntoIterator for Error {
    type Item = Error;
    type IntoIter = IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        match self.err {
            crate::gen::Error::Syn(err) => IntoIter::Syn(err.into_iter()),
            _ => IntoIter::Other(iter::once(self)),
        }
    }
}

pub enum IntoIter {
    Syn(<syn::Error as IntoIterator>::IntoIter),
    Other(iter::Once<Error>),
}

impl Iterator for IntoIter {
    type Item = Error;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            IntoIter::Syn(iter) => iter
                .next()
                .map(|syn_err| Error::from(crate::gen::Error::Syn(syn_err))),
            IntoIter::Other(iter) => iter.next(),
        }
    }
}

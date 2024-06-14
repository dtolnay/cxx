#![cfg(feature = "alloc")]

use alloc::boxed::Box;
use core::fmt::{self, Display};

#[cfg(all(feature = "std", not(no_core_error)))]
use core::error::Error as StdError;
#[cfg(all(feature = "std", no_core_error))]
use std::error::Error as StdError;

/// Exception thrown from an `extern "C++"` function.
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
#[derive(Debug)]
pub struct Exception {
    pub(crate) what: Box<str>,
}

impl Display for Exception {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&self.what)
    }
}

#[cfg(feature = "std")]
#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
impl StdError for Exception {}

impl Exception {
    #[allow(missing_docs)]
    pub fn what(&self) -> &str {
        &self.what
    }
}

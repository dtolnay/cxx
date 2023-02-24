#![cfg(feature = "alloc")]

use alloc::boxed::Box;
use core::fmt::{self, Display, Debug};

use crate::CxxException;

/// Exception thrown from an `extern "C++"` function.
#[cfg_attr(doc_cfg, doc(cfg(feature = "alloc")))]
pub struct Exception {
    pub(crate) src: CxxException,
    pub(crate) what: Box<str>,
}

impl Display for Exception {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&self.what)
    }
}

impl Debug for Exception {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Exception").field("what", &self.what).finish()
    }
}

#[cfg(feature = "std")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "std")))]
impl std::error::Error for Exception {}

impl Exception {
    #[allow(missing_docs)]
    pub fn what(&self) -> &str {
        &self.what
    }
}

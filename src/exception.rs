use std::fmt::{self, Debug, Display};

/// Exception thrown from an `extern "C"` function.
#[derive(Debug)]
pub struct Exception {
    pub(crate) what: Box<str>,
}

impl Display for Exception {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&self.what)
    }
}

impl std::error::Error for Exception {}

impl Exception {
    pub fn what(&self) -> &str {
        &self.what
    }
}

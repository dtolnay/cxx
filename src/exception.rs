use std::fmt::{self, Debug, Display};
use std::slice;

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

#[export_name = "cxxbridge02$exception"]
unsafe extern "C" fn exception(ptr: *const u8, len: usize) -> *const u8 {
    let slice = slice::from_raw_parts(ptr, len);
    let boxed = String::from_utf8_lossy(slice).into_owned().into_boxed_str();
    Box::leak(boxed).as_ptr()
}

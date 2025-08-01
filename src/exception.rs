use core::{
    ffi::CStr,
    fmt::{self, Display},
    ptr::NonNull,
};

#[cfg(error_in_core)]
use core::error::Error as StdError;
#[cfg(all(feature = "std", not(error_in_core)))]
use core::error::Error as StdError;

// Representation for kj::Exception* and functions to manipulated it,
pub(crate) mod repr {
    #[repr(C)]
    pub(crate) struct KjException {
        data: (),
    }

    extern "C" {
        #[link_name = "cxxbridge1$kjException$new"]
        pub fn new(
            ptr: *const u8,
            len: usize,
            file: *const u8,
            file_len: usize,
            line: i32,
        ) -> *mut KjException;

        #[link_name = "cxxbridge1$kjException$getDescription"]
        pub fn description(err: *mut KjException) -> *const i8;

        #[link_name = "cxxbridge1$kjException$dropInPlace"]
        pub fn drop_in_place(err: *mut KjException);
    }
}

/// Exception thrown from an `extern "C++"` function.
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
#[derive(Debug)]
pub struct Exception {
    pub(crate) err: NonNull<repr::KjException>,
}

unsafe impl Sync for Exception {}
unsafe impl Send for Exception {}

impl Display for Exception {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.what())
    }
}

#[cfg(any(error_in_core, feature = "std"))]
impl StdError for Exception {}

impl Exception {
    #[allow(missing_docs)]
    pub fn what(&self) -> &str {
        let description = unsafe { repr::description(self.err.as_ptr()) };
        unsafe {
            CStr::from_ptr(description)
                .to_str()
                .unwrap_or("bad kj::Exception description")
        }
    }
}

impl Drop for Exception {
    fn drop(&mut self) {
        unsafe { repr::drop_in_place(self.err.as_ptr()) }
    }
}

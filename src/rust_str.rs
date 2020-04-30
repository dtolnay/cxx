use std::mem;
use std::ptr::NonNull;
use std::slice;
use std::str;

// Not necessarily ABI compatible with &str. Codegen performs the translation.
#[repr(C)]
#[derive(Copy, Clone)]
pub struct RustStr {
    pub(crate) ptr: NonNull<u8>,
    pub(crate) len: usize,
}

impl RustStr {
    pub fn from(s: &str) -> Self {
        RustStr {
            ptr: NonNull::from(s).cast::<u8>(),
            len: s.len(),
        }
    }

    pub unsafe fn as_str<'a>(self) -> &'a str {
        let slice = slice::from_raw_parts(self.ptr.as_ptr(), self.len);
        str::from_utf8_unchecked(slice)
    }
}

const_assert_eq!(mem::size_of::<Option<RustStr>>(), mem::size_of::<RustStr>());

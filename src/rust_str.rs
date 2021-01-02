use core::mem;
use core::ptr::NonNull;
use core::str;

#[repr(C)]
pub struct RustStr {
    repr: NonNull<str>,
}

impl RustStr {
    pub fn from(repr: &str) -> Self {
        let repr = NonNull::from(repr);
        RustStr { repr }
    }

    pub unsafe fn as_str<'a>(self) -> &'a str {
        &*self.repr.as_ptr()
    }
}

const_assert_eq!(mem::size_of::<Option<RustStr>>(), mem::size_of::<RustStr>());

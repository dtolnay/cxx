use std::slice;
use std::str;

// Not necessarily ABI compatible with &str. Codegen performs the translation.
#[repr(C)]
#[derive(Copy, Clone)]
pub struct RustStr {
    ptr: *const u8,
    len: usize,
}

impl RustStr {
    pub fn from(s: &str) -> Self {
        RustStr {
            ptr: s.as_ptr(),
            len: s.len(),
        }
    }

    pub unsafe fn as_str<'a>(self) -> &'a str {
        let slice = slice::from_raw_parts(self.ptr, self.len);
        str::from_utf8_unchecked(slice)
    }
}

#[export_name = "cxxbridge01$rust_str$valid"]
unsafe extern "C" fn str_valid(ptr: *const u8, len: usize) -> bool {
    let slice = slice::from_raw_parts(ptr, len);
    str::from_utf8(slice).is_ok()
}

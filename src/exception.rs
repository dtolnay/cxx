use crate::rust_str::RustStr;
use std::fmt::Display;
use std::ptr;
use std::slice;
use std::str;

pub unsafe fn r#try<T, E>(ret: *mut T, result: Result<T, E>) -> Option<RustStr>
where
    E: Display,
{
    match result {
        Ok(ok) => {
            ptr::write(ret, ok);
            None
        }
        Err(err) => Some(to_c_string(err.to_string())),
    }
}

unsafe fn to_c_string(msg: String) -> RustStr {
    let mut msg = msg;
    msg.as_mut_vec().push(b'\0');
    let ptr = msg.as_ptr();
    let len = msg.len();

    extern "C" {
        #[link_name = "cxxbridge02$error"]
        fn error(ptr: *const u8, len: usize) -> *const u8;
    }

    let copy = error(ptr, len);
    let slice = slice::from_raw_parts(copy, len);
    let string = str::from_utf8_unchecked(slice);
    RustStr::from(string)
}

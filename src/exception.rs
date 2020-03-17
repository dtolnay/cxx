use std::fmt::Display;
use std::ptr;

#[repr(C)]
pub struct Error {
    ptr: *const u8,
    len: usize,
}

pub unsafe fn r#try<T, E>(ret: *mut T, result: Result<T, E>) -> Error
where
    E: Display,
{
    match result {
        Ok(ok) => {
            ptr::write(ret, ok);
            Error {
                ptr: ptr::null(),
                len: 0,
            }
        }
        Err(err) => to_c_string(err.to_string()),
    }
}

unsafe fn to_c_string(msg: String) -> Error {
    let mut msg = msg;
    msg.as_mut_vec().push(b'\0');
    let ptr = msg.as_ptr();
    let len = msg.len();

    extern "C" {
        #[link_name = "cxxbridge02$error"]
        fn error(ptr: *const u8, len: usize) -> *const u8;
    }

    let copy = error(ptr, len);
    Error { ptr: copy, len }
}

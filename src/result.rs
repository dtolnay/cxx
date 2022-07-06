#![cfg(feature = "alloc")]
#![allow(missing_docs)]

use crate::exception::Exception;
use alloc::boxed::Box;
use alloc::string::{String, ToString};
use core::ffi::c_void;
use core::fmt::Display;
use core::ptr::{self, NonNull};
use core::result::Result as StdResult;
use core::slice;
use core::str;

#[repr(C)]
#[derive(Copy, Clone)]
struct PtrLen {
    ptr: NonNull<u8>,
    len: usize,
    inner_err: *mut c_void,
    inner_destructor: extern "C" fn(*mut c_void),
}

#[repr(C)]
pub union Result {
    err: PtrLen,
    ok: *const u8, // null
}

extern "C" fn free_boxed_error<E>(ptr: *mut c_void) {
    if !ptr.is_null() {
        unsafe { std::mem::drop(Box::from_raw(ptr as *mut E)) }
    }
}

pub unsafe fn r#try<T, E>(ret: *mut T, result: StdResult<T, E>) -> Result
where
    E: Display,
{
    match result {
        Ok(ok) => {
            unsafe { ptr::write(ret, ok) }
            Result { ok: ptr::null() }
        }
        Err(err) => {
            let msg = err.to_string();
            let boxed_error = Box::new(err);
            let destructor = free_boxed_error::<E>;
            unsafe {
                to_c_error(
                    msg,
                    Box::leak(boxed_error) as *mut E as *mut c_void,
                    destructor,
                )
            }
        }
    }
}

unsafe fn to_c_error(msg: String, inner_err: *mut c_void, destructor: extern "C" fn(*mut c_void)) -> Result {
    let mut msg = msg;
    unsafe { msg.as_mut_vec() }.push(b'\0');
    let ptr = msg.as_ptr();
    let len = msg.len();

    extern "C" {
        #[link_name = "cxxbridge1$error"]
        fn error(ptr: *const u8, len: usize) -> NonNull<u8>;
    }

    let copy = unsafe { error(ptr, len) };
    let err = PtrLen {
        ptr: copy,
        len,
        inner_err,
        inner_destructor: destructor,
    };
    Result { err }
}

impl Result {
    pub unsafe fn exception(self) -> StdResult<(), Exception> {
        unsafe {
            if self.ok.is_null() {
                Ok(())
            } else {
                let err = self.err;
                let slice = slice::from_raw_parts_mut(err.ptr.as_ptr(), err.len);
                let s = str::from_utf8_unchecked_mut(slice);
                Err(Exception {
                    what: Box::from_raw(s),
                })
            }
        }
    }
}

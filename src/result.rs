#![cfg(feature = "alloc")]
#![allow(missing_docs)]

use crate::exception::Exception;
use alloc::boxed::Box;
use alloc::string::{String, ToString};
use core::fmt::Display;
use core::ptr::{self, NonNull};
use core::result::Result as StdResult;
use core::slice;
use core::str;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct PtrLen {
    pub ptr: NonNull<u8>,
    pub len: usize,
}

extern "C" {
    /// Helper to construct the default exception from the error message.
    #[link_name = "cxxbridge1$default_exception"]
    fn default_exception(ptr: *const u8, len: usize) -> *mut u8;
    /// Helper to clone the instance of `std::exception_ptr` on the C++ side.
    #[link_name = "cxxbridge1$clone_exception"]
    fn clone_exception(ptr: *const u8) -> *mut u8;
    /// Helper to drop the instance of `std::exception_ptr` on the C++ side.
    #[link_name = "cxxbridge1$drop_exception"]
    fn drop_exception(ptr: *mut u8);
}

/// C++ exception containing `std::exception_ptr`.
///
/// This object is the Rust wrapper over `std::exception_ptr`, so it owns the exception pointer.
/// I.e., the exception is either referenced by a `std::exception_ptr` on the C++ side or the
/// reference is moved to this object on the Rust side.
#[repr(C)]
#[must_use]
pub struct CxxException(NonNull<u8>);

impl CxxException {
    /// Construct the default `rust::Error` exception from the specified `exc_text`.
    pub fn new_default(exc_text: &str) -> Self {
        let exception_ptr = unsafe {
            default_exception(exc_text.as_ptr(), exc_text.len())
        };
        CxxException(
            NonNull::new(exception_ptr)
            .expect("Exception conversion returned a null pointer")
        )
    }
}

impl Clone for CxxException {
    fn clone(&self) -> Self {
        let clone_ptr = unsafe { clone_exception(self.0.as_ptr()) };
        Self(
            NonNull::new(clone_ptr)
            .expect("Exception cloning returned a null pointer")
        )
    }
}

impl Drop for CxxException {
    fn drop(&mut self) {
        unsafe { drop_exception(self.0.as_ptr()) };
    }
}

// SAFETY: This is safe, since the C++ exception referenced by `std::exception_ptr`
// is not thread-local.
unsafe impl Send for CxxException {}
// SAFETY: This is safe, since the C++ exception referenced by `std::exception_ptr`
// can be shared across threads read-only.
unsafe impl Sync for CxxException {}

#[repr(C)]
pub union Result {
    err: PtrLen,
    ok: *const u8, // null
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
        Err(err) => unsafe { to_c_error(err.to_string()) },
    }
}

unsafe fn to_c_error(msg: String) -> Result {
    let mut msg = msg;
    unsafe { msg.as_mut_vec() }.push(b'\0');
    let ptr = msg.as_ptr();
    let len = msg.len();

    extern "C" {
        #[link_name = "cxxbridge1$error"]
        fn error(ptr: *const u8, len: usize) -> NonNull<u8>;
    }

    let copy = unsafe { error(ptr, len) };
    let err = PtrLen { ptr: copy, len };
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

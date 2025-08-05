#![allow(missing_docs)]

use crate::exception::repr;
use crate::exception::Exception;
use alloc::string::ToString;
use core::fmt::Display;
use core::ptr::{self, NonNull};
use core::result::Result as StdResult;

#[repr(C)]
pub struct Result {
    err: *mut repr::KjException,
}

impl Result {
    pub(crate) fn ok() -> Self {
        Result {
            err: ptr::null_mut(),
        }
    }

    pub(crate) fn error(msg: &str, file: &str, line: u32) -> Self {
        let err = unsafe {
            repr::new(
                msg.as_ptr(),
                msg.len(),
                file.as_ptr(),
                file.len(),
                line.try_into().unwrap_or_default(),
            )
        };
        Self { err }
    }
}

pub unsafe fn r#try<T, E>(ret: *mut T, result: StdResult<T, E>, file: &str, line: u32) -> Result
where
    E: Display,
{
    match result {
        Ok(ok) => {
            unsafe { ptr::write(ret, ok) }
            Result::ok()
        }
        Err(err) => Result::error(&err.to_string(), file, line),
    }
}

impl Result {
    pub unsafe fn exception(self) -> StdResult<(), Exception> {
        let err = self.err;
        core::mem::forget(self);
        match NonNull::new(err) {
            Some(err) => Err(Exception { err }),
            None => Ok(()),
        }
    }
}

impl Drop for Result {
    fn drop(&mut self) {
        unsafe { repr::drop_in_place(self.err) }
    }
}

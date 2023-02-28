#![cfg(feature = "alloc")]
#![allow(missing_docs)]

use alloc::boxed::Box;
use alloc::string::ToString;
use core::fmt::Display;
#[cfg(feature = "std")]
use core::mem::MaybeUninit;
use core::ptr::NonNull;
use core::slice;
use core::str;

use crate::Exception;

#[repr(C)]
#[derive(Copy, Clone)]
pub(crate) struct PtrLen {
    pub ptr: NonNull<u8>,
    pub len: usize,
}

/// Representation of C++ `std::exception_ptr` for all targets except MSVC.
///
/// This is a single pointer.
#[repr(C)]
#[derive(Copy, Clone)]
#[cfg(not(target_env = "msvc"))]
struct CxxExceptionRepr {
    ptr: NonNull<u8>,
}

/// Representation of C++ `std::exception_ptr` for MSVC.
///
/// Unfortunately, MSVC uses two pointers for `std::exception_ptr`, so we have
/// to account for that.
#[repr(C)]
#[derive(Copy, Clone)]
#[cfg(target_env = "msvc")]
struct CxxExceptionRepr {
    ptr: NonNull<u8>,
    _ptr2: *mut u8,
}

extern "C" {
    /// Helper to construct the default exception from the error message.
    #[link_name = "cxxbridge1$default_exception"]
    fn default_exception(ptr: *const u8, len: usize) -> CxxExceptionRepr;
    /// Helper to clone the instance of `std::exception_ptr` on the C++ side.
    #[link_name = "cxxbridge1$clone_exception"]
    fn clone_exception(ptr: &CxxExceptionRepr) -> CxxExceptionRepr;
    /// Helper to drop the instance of `std::exception_ptr` on the C++ side.
    #[link_name = "cxxbridge1$drop_exception"]
    fn drop_exception(ptr: CxxExceptionRepr);
}

/// C++ exception containing an `std::exception_ptr`.
///
/// This object is the Rust wrapper over `std::exception_ptr`, so it owns the exception pointer.
/// I.e., the exception is either referenced by a `std::exception_ptr` on the C++ side or the
/// reference is moved to this object on the Rust side.
#[repr(C)]
#[must_use]
pub struct CxxException(CxxExceptionRepr);

impl CxxException {
    /// Construct the default `rust::Error` exception from the specified `exc_text`.
    fn new_default(exc_text: &str) -> Self {
        let exception_repr = unsafe { default_exception(exc_text.as_ptr(), exc_text.len()) };
        CxxException(exception_repr)
    }
}

impl Clone for CxxException {
    fn clone(&self) -> Self {
        let exception_repr = unsafe { clone_exception(&self.0) };
        Self(exception_repr)
    }
}

impl From<Exception> for CxxException {
    fn from(value: Exception) -> Self {
        value.src
    }
}

impl Drop for CxxException {
    fn drop(&mut self) {
        unsafe { drop_exception(self.0) };
    }
}

// SAFETY: This is safe, since the C++ exception referenced by `std::exception_ptr`
// is not thread-local.
unsafe impl Send for CxxException {}
// SAFETY: This is safe, since the C++ exception referenced by `std::exception_ptr`
// can be shared across threads read-only.
unsafe impl Sync for CxxException {}

/// C++ "result" containing `std::exception_ptr` or a `null`.
#[repr(C)]
pub struct CxxResult(Option<CxxException>);

impl From<CxxException> for CxxResult {
    fn from(value: CxxException) -> Self {
        Self(Some(value))
    }
}

impl CxxResult {
    /// Construct an empty `Ok` result.
    pub fn new() -> Self {
        Self(None)
    }
}

impl CxxResult {
    unsafe fn exception(self) -> Result<(), CxxException> {
        // SAFETY: We know that the `Result` can only contain a valid `std::exception_ptr` or null.
        match self.0 {
            None => Ok(()),
            Some(ptr) => Err(ptr),
        }
    }
}

// Assert that the result is not larger than the exception (`Option` will use the niche).
const _: () = assert!(core::mem::size_of::<CxxResult>() == core::mem::size_of::<CxxException>());

#[repr(C)]
pub struct CxxResultWithMessage {
    pub(crate) res: CxxResult,
    pub(crate) msg: PtrLen,
}

impl CxxResultWithMessage {
    pub unsafe fn exception(self) -> Result<(), Exception> {
        // SAFETY: We know that the `Result` can only contain a valid `std::exception_ptr` or null.
        match unsafe { self.res.exception() } {
            Ok(_) => Ok(()),
            Err(src) => {
                // SAFETY: The message is always given for the exception and we constructed it in
                // a `Box` in `cxxbridge1$exception()`. We just reconstruct it here.
                let what = unsafe {
                    str::from_utf8_unchecked_mut(slice::from_raw_parts_mut(
                        self.msg.ptr.as_ptr(),
                        self.msg.len,
                    ))
                };
                Err(Exception {
                    src,
                    what: unsafe { Box::from_raw(what) },
                })
            }
        }
    }
}

/// Trait to convert an arbitrary Rust error into a C++ exception.
///
/// If an implementation of [`ToCxxException`] is explicitly provided for an `E`, then this
/// implementation will be used for an `extern "Rust"` function returning a `Result<_, E>`.
/// The implementation will likely call back to C++ to create the `exception_ptr` based on
/// some parameters of the Rust error.
///
/// The default implementation is implemented in a second trait [`ToCxxExceptionDefault`]
/// to work around Rust limitations (missing specialization in stable Rust). It creates
/// a C++ exception of the type `rust::Error` with the text of the Rust exception serialized
/// via `E::to_string()` (unless overridden via [`set_exception_handler()`]).
pub trait ToCxxException {
    /// Specific conversion implementation for `Self`.
    fn to_cxx_exception(&self) -> CxxException;
}

/// Default implementation for converting errors to C++ exceptions for types not implementing
/// [`ToCxxException`].
///
/// Do not implement this trait. Implement [`ToCxxException`] for `E` instead to customize
/// `Result<_, E>` handling in an `extern "Rust"` function.
pub trait ToCxxExceptionDefault {
    fn to_cxx_exception(&self) -> CxxException;
}

// Identity conversion for an existing C++ exception.
impl ToCxxException for CxxException {
    fn to_cxx_exception(&self) -> CxxException {
        self.clone()
    }
}

// Default conversion for errors with a message.
impl<T: Display> ToCxxExceptionDefault for &T {
    fn to_cxx_exception(&self) -> CxxException {
        #[cfg(feature = "std")]
        {
            // In order to prevent yet another allocation(s) for the string, first
            // try to materialize the error message in an on-stack buffer.
            const INLINE_BUFFER_SIZE: usize = 4096;

            let mut buffer = MaybeUninit::<[u8; INLINE_BUFFER_SIZE]>::uninit();
            let size = {
                use std::io::Write;
                let buffer: &mut [u8] = unsafe { buffer.assume_init_mut() };
                let mut cursor = std::io::Cursor::new(buffer);
                let res = write!(cursor, "{self}");
                if res.is_err() {
                    // the buffer was insufficient, allocate a string
                    let exc_text = self.to_string();
                    return CxxException::new_default(&exc_text);
                }
                cursor.position() as usize
            };
            // we have sufficient buffer size, just construct from the inplace
            // buffer
            let exc_text =
                unsafe { std::str::from_utf8_unchecked(&buffer.assume_init_ref()[0..size]) };
            CxxException::new_default(exc_text)
        }
        #[cfg(not(feature = "std"))]
        {
            // no Cursor available in no-std case
            let exc_text = self.to_string();
            return CxxException::new_default(&exc_text);
        }
    }
}

#[macro_export]
macro_rules! map_rust_error_to_cxx_exception {
    ($err:ident) => {{
        #[allow(unused_imports)]
        let exc = {
            // NOTE: This trick helps us to specialize exception generation for error types without
            // the need for `specialization` feature. Namely, `ToCxxException` for `T` has higher
            // weight and is selected before `ToCxxExceptionDefault`, which is defined on `&T` (and
            // requires auto-deref). If it's not defined, then the default is used.
            use $crate::ToCxxException;
            use $crate::ToCxxExceptionDefault;
            (&$err).to_cxx_exception()
        };
        exc
    }};
}

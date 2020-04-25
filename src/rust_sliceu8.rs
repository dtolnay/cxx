use std::mem;
use std::ptr::NonNull;
use std::slice;

// Not necessarily ABI compatible with &[u8]. Codegen performs the translation.
#[repr(C)]
#[derive(Copy, Clone)]
pub struct RustSliceU8 {
    pub(crate) ptr: NonNull<u8>,
    pub(crate) len: usize,
}

impl RustSliceU8 {
    pub fn from(s: &[u8]) -> Self {
        RustSliceU8 {
            ptr: NonNull::from(s).cast::<u8>(),
            len: s.len(),
        }
    }

    pub unsafe fn as_slice<'a>(self) -> &'a [u8] {
        slice::from_raw_parts(self.ptr.as_ptr(), self.len)
    }
}

const_assert_eq!(
    mem::size_of::<Option<RustSliceU8>>(),
    mem::size_of::<RustSliceU8>(),
);

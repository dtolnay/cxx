use core::mem;
use core::ptr::NonNull;
use core::slice;

// Not necessarily ABI compatible with &mut [u8]. Codegen performs the translation.
#[repr(C)]
#[derive(Copy, Clone)]
pub struct RustMutSliceU8 {
    pub(crate) ptr: NonNull<u8>,
    pub(crate) len: usize,
}

impl RustMutSliceU8 {
    pub fn from(s: &mut [u8]) -> Self {
        let len = s.len();
        RustMutSliceU8 {
            ptr: NonNull::from(s).cast::<u8>(),
            len,
        }
    }

    pub unsafe fn as_mut_slice<'a>(self) -> &'a mut [u8] {
        slice::from_raw_parts_mut(self.ptr.as_ptr(), self.len)
    }
}

const_assert_eq!(
    mem::size_of::<Option<RustMutSliceU8>>(),
    mem::size_of::<RustMutSliceU8>(),
);

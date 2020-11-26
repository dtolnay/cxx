use core::mem;
use core::ptr::NonNull;
use core::slice;

// Not necessarily ABI compatible with &[T]. Codegen performs the translation.
#[repr(C)]
#[derive(Copy, Clone)]
pub struct RustSlice {
    pub(crate) ptr: NonNull<()>,
    pub(crate) len: usize,
}

impl RustSlice {
    pub fn from_ref<T>(s: &[T]) -> Self {
        RustSlice {
            ptr: NonNull::from(s).cast::<()>(),
            len: s.len(),
        }
    }

    pub fn from_mut<T>(s: &mut [T]) -> Self {
        RustSlice {
            len: s.len(),
            ptr: NonNull::from(s).cast::<()>(),
        }
    }

    pub unsafe fn as_slice<'a, T>(self) -> &'a [T] {
        slice::from_raw_parts(self.ptr.as_ptr().cast::<T>(), self.len)
    }

    pub unsafe fn as_mut_slice<'a, T>(self) -> &'a mut [T] {
        slice::from_raw_parts_mut(self.ptr.as_ptr().cast::<T>(), self.len)
    }
}

const_assert_eq!(
    mem::size_of::<Option<RustSlice>>(),
    mem::size_of::<RustSlice>(),
);

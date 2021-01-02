use core::mem;
use core::ptr::{self, NonNull};
use core::slice;

#[repr(C)]
pub struct RustSlice {
    pub(crate) repr: NonNull<[()]>,
}

impl RustSlice {
    pub fn from_ref<T>(slice: &[T]) -> Self {
        let ptr = ptr::slice_from_raw_parts::<()>(slice.as_ptr().cast(), slice.len());
        RustSlice {
            repr: unsafe { NonNull::new_unchecked(ptr as *mut _) },
        }
    }

    pub fn from_mut<T>(slice: &mut [T]) -> Self {
        let ptr = ptr::slice_from_raw_parts_mut(slice.as_mut_ptr().cast(), slice.len());
        RustSlice {
            repr: unsafe { NonNull::new_unchecked(ptr) },
        }
    }

    pub unsafe fn as_slice<'a, T>(self) -> &'a [T] {
        let ptr = self.repr.as_ptr();
        let len = self.repr.as_ref().len();
        slice::from_raw_parts(ptr.cast(), len)
    }

    pub unsafe fn as_mut_slice<'a, T>(self) -> &'a mut [T] {
        let ptr = self.repr.as_ptr();
        let len = self.repr.as_ref().len();
        slice::from_raw_parts_mut(ptr.cast(), len)
    }
}

const_assert_eq!(
    mem::size_of::<Option<RustSlice>>(),
    mem::size_of::<RustSlice>(),
);

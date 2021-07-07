#![allow(missing_docs)]

use core::mem::{ManuallyDrop, MaybeUninit};

union OptionInner<T> {
    value: ManuallyDrop<MaybeUninit<T>>,
    empty: usize,
}

// ABI compatible with C++ rust::Option<T> (not necessarily core::option::Option<T>).
#[repr(C)]
pub struct RustOption<T> {
    inner: OptionInner<T>,
}

impl<T> RustOption<T> {
    pub fn new() -> Self {
        Self::from(None)
    }

    pub fn value(&self) -> Option<&T> {
        if unsafe { self.inner.empty != 0 } {
            unsafe { self.inner.value.as_ptr().as_ref() }
        } else {
            None
        }
    }

    pub fn from(option: Option<T>) -> Self {
        match option {
            Some(value) => RustOption {
                inner: OptionInner{ value: ManuallyDrop::new(MaybeUninit::new(value)) },
            },
            None => RustOption {
                inner: OptionInner{ empty: 0 },
            },
        }
    }

    pub fn into_option(self) -> Option<T> {
        if unsafe { self.inner.empty != 0 } {
            Some(unsafe { self.into_inner_unchecked() })
        } else {
            None
        }
    }

    pub unsafe fn into_inner_unchecked(mut self) -> T {
        ManuallyDrop::take(&mut self.inner.value).assume_init()
    }
}

impl<T> Drop for RustOption<T> {
    fn drop(&mut self) {
        if unsafe { self.inner.empty != 0 } {
            unsafe { ManuallyDrop::drop(&mut self.inner.value) }
        }
    }
}
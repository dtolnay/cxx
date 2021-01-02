use crate::rust_slice::RustSlice;
use core::mem::MaybeUninit;
use core::ptr::{self, NonNull};

#[export_name = "cxxbridge1$slice$new"]
unsafe extern "C" fn slice_new(this: &mut MaybeUninit<RustSlice>, ptr: *const (), len: usize) {
    let ptr = ptr::slice_from_raw_parts(ptr, len);
    let rust_slice = RustSlice {
        repr: NonNull::new_unchecked(ptr as *mut _),
    };
    ptr::write(this.as_mut_ptr(), rust_slice);
}

#[export_name = "cxxbridge1$slice$ptr"]
unsafe extern "C" fn slice_ptr(this: &RustSlice) -> *const () {
    this.repr.as_ptr().cast()
}

#[export_name = "cxxbridge1$slice$len"]
unsafe extern "C" fn slice_len(this: &RustSlice) -> usize {
    this.repr.as_ref().len()
}

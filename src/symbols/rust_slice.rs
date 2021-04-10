use crate::rust_slice::RustSlice;
use core::mem::MaybeUninit;
use core::ptr::{self, NonNull};

#[export_name = "cxxbridge1$slice$new"]
unsafe extern "C" fn slice_new(this: &mut MaybeUninit<RustSlice>, ptr: NonNull<()>, len: usize) {
    let rust_slice = RustSlice::from_raw_parts(ptr, len);
    ptr::write(this.as_mut_ptr(), rust_slice);
}

#[export_name = "cxxbridge1$slice$ptr"]
unsafe extern "C" fn slice_ptr(this: &RustSlice) -> NonNull<()> {
    this.as_ptr()
}

#[export_name = "cxxbridge1$slice$len"]
unsafe extern "C" fn slice_len(this: &RustSlice) -> usize {
    this.len()
}

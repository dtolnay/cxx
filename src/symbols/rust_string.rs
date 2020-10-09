use alloc::borrow::ToOwned;
use alloc::string::String;
use core::mem::{ManuallyDrop, MaybeUninit};
use core::ptr;
use core::slice;
use core::str;

#[export_name = "cxxbridge05$string$new"]
unsafe extern "C" fn string_new(this: &mut MaybeUninit<String>) {
    ptr::write(this.as_mut_ptr(), String::new());
}

#[export_name = "cxxbridge05$string$clone"]
unsafe extern "C" fn string_clone(this: &mut MaybeUninit<String>, other: &String) {
    ptr::write(this.as_mut_ptr(), other.clone());
}

#[export_name = "cxxbridge05$string$from"]
unsafe extern "C" fn string_from(
    this: &mut MaybeUninit<String>,
    ptr: *const u8,
    len: usize,
) -> bool {
    let slice = slice::from_raw_parts(ptr, len);
    match str::from_utf8(slice) {
        Ok(s) => {
            ptr::write(this.as_mut_ptr(), s.to_owned());
            true
        }
        Err(_) => false,
    }
}

#[export_name = "cxxbridge05$string$drop"]
unsafe extern "C" fn string_drop(this: &mut ManuallyDrop<String>) {
    ManuallyDrop::drop(this);
}

#[export_name = "cxxbridge05$string$ptr"]
unsafe extern "C" fn string_ptr(this: &String) -> *const u8 {
    this.as_ptr()
}

#[export_name = "cxxbridge05$string$len"]
unsafe extern "C" fn string_len(this: &String) -> usize {
    this.len()
}

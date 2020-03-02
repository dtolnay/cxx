use std::mem::{self, ManuallyDrop, MaybeUninit};
use std::ptr;
use std::slice;
use std::str;

#[repr(C)]
pub struct RustString {
    repr: String,
}

impl RustString {
    pub fn from(s: String) -> Self {
        RustString { repr: s }
    }

    pub fn from_ref(s: &String) -> &Self {
        unsafe { std::mem::transmute::<&String, &RustString>(s) }
    }

    pub fn into_string(self) -> String {
        self.repr
    }

    pub fn as_string(&self) -> &String {
        &self.repr
    }
}

#[export_name = "cxxbridge01$string$new"]
unsafe extern "C" fn string_new(this: &mut MaybeUninit<String>) {
    ptr::write(this.as_mut_ptr(), String::new());
}

#[export_name = "cxxbridge01$string$clone"]
unsafe extern "C" fn string_clone(this: &mut MaybeUninit<String>, other: &String) {
    ptr::write(this.as_mut_ptr(), other.clone());
}

#[export_name = "cxxbridge01$string$from"]
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

#[export_name = "cxxbridge01$string$drop"]
unsafe extern "C" fn string_drop(this: &mut ManuallyDrop<String>) {
    ManuallyDrop::drop(this);
}

#[export_name = "cxxbridge01$string$ptr"]
unsafe extern "C" fn string_ptr(this: &String) -> *const u8 {
    this.as_ptr()
}

#[export_name = "cxxbridge01$string$len"]
unsafe extern "C" fn string_len(this: &String) -> usize {
    this.len()
}

fn _assert() {
    let _: [(); mem::size_of::<[usize; 3]>()] = [(); mem::size_of::<String>()];
    let _: [(); mem::align_of::<usize>()] = [(); mem::align_of::<String>()];
}

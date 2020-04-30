use std::slice;
use std::str;

#[export_name = "cxxbridge03$str$valid"]
unsafe extern "C" fn str_valid(ptr: *const u8, len: usize) -> bool {
    let slice = slice::from_raw_parts(ptr, len);
    str::from_utf8(slice).is_ok()
}

use std::slice;

#[export_name = "cxxbridge03$exception"]
unsafe extern "C" fn exception(ptr: *const u8, len: usize) -> *const u8 {
    let slice = slice::from_raw_parts(ptr, len);
    let boxed = String::from_utf8_lossy(slice).into_owned().into_boxed_str();
    Box::leak(boxed).as_ptr()
}

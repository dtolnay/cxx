#[cxx::bridge]
pub mod ffi {
    unsafe extern "C++" {
        include!("repro/helper.h");
        fn SendKey(key: c_char) -> u32;
    }
}

#[unsafe(no_mangle)]
pub fn function1() {
    ffi::SendKey(104);
    ffi::SendKey(105);
}

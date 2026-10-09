#[cxx::bridge]
pub mod ffi {
    extern "Rust" {
        // functions exported by the cdylib, imported by test exe
        fn get_magic_number() -> i32;
        fn multiply_values(a: i32, b: i32) -> i32;
        fn library_entry_point() -> i32;
    }

    unsafe extern "C++" {
        include!("exe_functions.h");

        // functions exported by test exe, imported by cdylib
        fn exe_callback(value: i32) -> i32;
        fn exe_get_constant() -> i32;
    }
}

pub fn get_magic_number() -> i32 {
    // call back to the exe to get a constant, then add our magic
    let exe_value = ffi::exe_get_constant();
    exe_value + 42
}

pub fn multiply_values(a: i32, b: i32) -> i32 {
    // use exe callback to process the result
    let product = a * b;
    ffi::exe_callback(product)
}

pub fn library_entry_point() -> i32 {
    // test that we can call exe functions from the library
    ffi::exe_callback(100) + ffi::exe_get_constant()
}

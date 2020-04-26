use std::os::raw::{c_char};

extern "C" {fn cpp_function(name: *const c_char);}

fn greeting(name: &str) {
    let name = std::ffi::CString::new(name).unwrap();
    unsafe {
        cpp_function(name.as_ptr());
    }
}

fn main() {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    if args.len() >= 1 {
        greeting(&args[0]);
    } else {
        greeting("Rust");
    }
}

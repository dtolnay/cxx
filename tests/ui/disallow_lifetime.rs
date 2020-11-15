#[cxx::bridge]
mod ffi {
    unsafe extern "C++" {
        type C;
        fn f(&'static self);
    }

    extern "Rust" {
        fn f(string: &'a String);
    }
}

fn main() {}

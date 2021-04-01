#[cxx::bridge]
mod ffi {
    unsafe extern "C++" {
        include!("smessmer/include/roundtrip.h");
        fn cppfunc(v: &MyCustomType);
    }

    extern "Rust" {
        type MyCustomType;
        fn rustfunc(v: &MyCustomType);
    }
}

pub struct MyCustomType {}

fn rustfunc(v: &MyCustomType) {
    let _ = v;
    println!("success");
}

fn main() {
    let v = MyCustomType {};
    ffi::cppfunc(&v);
}

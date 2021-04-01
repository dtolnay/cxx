use smessmer::MyCustomType;

#[cxx::bridge]
mod ffi {
    unsafe extern "C++" {
        include!("smessmer/include/roundtrip.h");
        type MyCustomType = smessmer::MyCustomType;
        fn cppfunc(v: &MyCustomType);
    }
}

fn main() {
    let v = MyCustomType {};
    ffi::cppfunc(&v);
}

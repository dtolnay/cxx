#[cxx::bridge]
mod ffi {
    unsafe extern "C++" {
        fn arraystr() -> [String; "13"];
        fn arraysub() -> [String; 15 - 1];
    }
}

fn main() {}

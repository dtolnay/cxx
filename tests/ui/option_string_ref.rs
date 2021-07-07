#[cxx::bridge]
mod ffi {
    extern "Rust" {
        fn f() -> Option<&'static String>;
    }
}

fn f() -> Option<&'static String> {
    unimplemented!()
}

fn main() {}

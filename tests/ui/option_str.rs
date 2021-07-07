#[cxx::bridge]
mod ffi {
    extern "Rust" {
        fn f() -> Option<&'static str>;
    }
}

fn f() -> Option<&'static str> {
    unimplemented!()
}

fn main() {}

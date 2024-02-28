#[cxx::bridge]
mod ffi {
    extern "Rust" {
        fn f() -> Option<String>;
    }
}

fn f() -> Option<String> {
    unimplemented!()
}

fn main() {}

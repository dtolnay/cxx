#[cxx::bridge]
mod ffi {
    extern "Rust" {
        fn f() -> Option<Vec<u8>>;
    }
}

fn f() -> Option<Vec<u8>> {
    unimplemented!()
}

fn main() {}

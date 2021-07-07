#[cxx::bridge]
mod ffi {
    extern "Rust" {
        fn f() -> Option<&'static Vec<u8>>;
    }
}

fn f() -> Option<&'static Vec<u8>> {
    unimplemented!()
}

fn main() {}

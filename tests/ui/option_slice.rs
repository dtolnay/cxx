#[cxx::bridge]
mod ffi {
    extern "Rust" {
        fn f() -> Option<&'static [u8]>;
    }
}

fn f() -> Option<&'static [u8]> {
    unimplemented!()
}

fn main() {}

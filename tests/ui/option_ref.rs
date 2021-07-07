#[cxx::bridge]
mod ffi {
    extern "Rust" {
        fn f(_o: &Option<&u8>);
    }
}

fn f(_o: &Option<&u8>) {}

fn main() {}

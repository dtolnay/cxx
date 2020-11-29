pub struct RustType;

#[cxx::bridge]
mod ffi {
    extern "Rust" {
        type RustType;
        fn new_client() -> Box<RustType>;
    }
}

fn new_client() -> Box<RustType> {
    Box::new(RustType)
}

#[cxx::bridge]
mod ffi {
    extern "Rust" {
        type T;
        fn t_method(&self);
        fn t_method(&self);
    }
}

fn main() {}

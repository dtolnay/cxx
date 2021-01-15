#[cxx::bridge]
mod ffi {
    extern "Rust" {
        fn self_rust(self);
    }

    extern "C++" {
        fn self_c(self);
    }
}

fn main() {}

#[cxx::bridge]
mod ffi {
    struct UniquePtr {
        // invalid; `UniquePtr` is a builtin
        val: usize,
    }

    extern "C++" {
        type Box; // invalid; `Box` is a builtin
    }

    extern "Rust" {
        type String; // valid; `String` is an atom
    }
}

fn main() {}

struct Type;

#[cxx::bridge]
mod ffi {
    extern "Rust" {
        // Not allowed - the target is not a path.
        type Source = &crate::Type;
    }
}

fn main() {}

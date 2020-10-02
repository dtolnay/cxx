// Rustfmt mangles the extern type alias.
// https://github.com/rust-lang/rustfmt/issues/4159
// ...normally would add #[rustfmt::skip], but that seems to interfere with the error spans.
#[cxx::bridge]
mod ffi {
    extern "C" {
        fn unsupported_foreign_item() {}

        type BadGeneric<T> = Bad;
    }

    extern "Rust" {
        /// Incorrect.
        type Alias = crate::Type;
    }
}

fn main() {}

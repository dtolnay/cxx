#[cxx::bridge]
mod here {
    struct Shared {
        z: usize,
    }

    extern "C" {
        type C;
    }
}

// Rustfmt mangles the extern type alias.
// https://github.com/rust-lang/rustfmt/issues/4159
#[rustfmt::skip]
#[cxx::bridge]
mod there {
    type C = crate::here::C;

    extern "C" {
        type Shared = crate::here::Shared;
    }
}

fn main() {}

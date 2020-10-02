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
// ...normally would add #[rustfmt::skip], but that seems to interfere with the error spans.
#[cxx::bridge]
mod there {
    type Shared = crate::here::Shared;

    extern "C" {
        type C = crate::here::C;

        fn good(self: &C);
        fn bad(self: &Shared);
    }
}

fn main() {}

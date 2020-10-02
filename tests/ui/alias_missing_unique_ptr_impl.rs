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
    type Shared = crate::here::Shared;

    extern "C" {
        type C = crate::here::C;

        fn c_take_unique_ptr(s: UniquePtr<C>);
        fn c_take_unique_ptr_shared(s: UniquePtr<Shared>);
    }
}

fn main() {}

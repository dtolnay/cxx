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

        fn c_take_unique_ptr_vector(s: UniquePtr<CxxVector<C>>);
        fn c_take_unique_ptr_vector_shared(s: UniquePtr<CxxVector<Shared>>);
    }
}

fn main() {}

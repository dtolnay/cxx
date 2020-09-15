// This fails because the original bridge does not see any usage of
// UniquePtr<C> and so does not generate the approprite trait. The
// original bridge currently needs to include some usage of UniquePtr<C>.
// TODO: File a Github issue to fix this.

#[cxx::bridge(namespace = here)]
mod here {
    extern "C" {
        type C;
    }
}

// Rustfmt mangles the extern type alias.
// https://github.com/rust-lang/rustfmt/issues/4159
#[rustfmt::skip]
#[cxx::bridge(namespace = there)]
#[cxx::alias_namespace(crate::here = here)]
mod there {
    extern "C" {
        type C = crate::here::C;

        fn c_return_unique_ptr() -> UniquePtr<C>;
    }
}

fn main() {}

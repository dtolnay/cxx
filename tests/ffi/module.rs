// Rustfmt mangles the extern type alias.
// https://github.com/rust-lang/rustfmt/issues/4159
#[rustfmt::skip]
#[cxx::bridge(namespace = "tests")]
pub mod ffi {
    extern "C" {
        include!("tests/ffi/tests.h");

        type C = crate::ffi::C;

        fn c_take_unique_ptr(c: UniquePtr<C>);
    }
}

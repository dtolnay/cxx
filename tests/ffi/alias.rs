// Rustfmt mangles the extern type alias.
// https://github.com/rust-lang/rustfmt/issues/4159
#[rustfmt::skip]
#[cxx::bridge(namespace = alias_tests)]
pub mod ffi {
    extern "C" {
        include!("cxx-test-suite/tests.h");

        // Review TODO: Unquoted namespace here doesn't work, is that expected or a bug
        // in my parsing?
        #[namespace = "tests"]
        type C = crate::ffi::C;

        fn c_return_unique_ptr() -> UniquePtr<C>;
        fn c_take_unique_ptr(c: UniquePtr<C>);
    }
}

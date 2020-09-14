// Rustfmt mangles the extern type alias.
// https://github.com/rust-lang/rustfmt/issues/4159
#[rustfmt::skip]
#[cxx::bridge(namespace = alias_tests)]
#[cxx::alias_namespace(crate::ffi = tests)]
pub mod ffi {
    extern "C" {
        include!("cxx-test-suite/tests.h");

        type C = crate::ffi::C;
        type SameC = crate::ffi::C;

        fn c_return_unique_ptr() -> UniquePtr<C>;
        fn c_take_unique_ptr(c: UniquePtr<SameC>);
    }
}

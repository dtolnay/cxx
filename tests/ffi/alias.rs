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

        // Used to test multiple cxx::alias_namespace attributes in alias2
        type DifferentC;

        fn c_return_unique_ptr() -> UniquePtr<C>;
        fn c_take_unique_ptr(c: UniquePtr<SameC>);

        // TODO: Workaround for github.com/dtolnay/cxx/issues/XXX
        fn create_different_c() -> UniquePtr<DifferentC>;
    }
}

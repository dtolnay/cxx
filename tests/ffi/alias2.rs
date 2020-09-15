// Rustfmt mangles the extern type alias.
// https://github.com/rust-lang/rustfmt/issues/4159
#[rustfmt::skip]
#[cxx::bridge(namespace = alias2_tests)]
#[cxx::alias_namespace(crate::ffi = tests)]
#[cxx::alias_namespace(crate::alias::ffi = alias_tests)]
pub mod ffi {
    extern "C" {
        include!("cxx-test-suite/tests.h");
        include!("cxx-test-suite/alias.rs.h");

        type C = crate::ffi::C;
        type DifferentC = crate::alias::ffi::DifferentC;

        fn c_return_unique_ptr() -> UniquePtr<C>;
        fn c_take_unique_ptr(c: UniquePtr<C>);

        fn create_different_c() -> UniquePtr<DifferentC>;
    }
}

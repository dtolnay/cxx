// Separate mod so that &self in the lib.rs mod has an unambiguous receiver. At
// the moment, the cxx C++ codegen can't convert more than one cxx::bridge mod
// per file, so that's why we need to put this outside of lib.rs. All of this
// could go into module.rs instead, but for now its purpose is narrowly scoped
// for testing aliasing between cxx::bridge mods, so we'll keep it that way and
// start a new mod here.

// Rustfmt mangles the extern type alias.
// https://github.com/rust-lang/rustfmt/issues/4159
#[rustfmt::skip]
#[cxx::bridge(namespace = tests)]
pub mod ffi2 {
    impl UniquePtr<D> {}
    impl UniquePtr<E> {}

    extern "C" {
        include!("tests/ffi/tests.h");

        type D = crate::other::D;
        type E = crate::other::E;

        fn c_take_trivial_ptr(d: UniquePtr<D>);
        fn c_take_trivial_ref(d: &D);
        fn c_take_trivial(d: D);
        fn c_take_opaque_ptr(e: UniquePtr<E>);
        fn c_take_opaque_ref(e: &E);
        fn c_return_trivial_ptr() -> UniquePtr<D>;
        fn c_return_trivial() -> D;
        fn c_return_opaque_ptr() -> UniquePtr<E>;
    }
}

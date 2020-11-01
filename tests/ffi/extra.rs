// Separate mod so that &self in the lib.rs mod has an unambiguous receiver. At
// the moment, the cxx C++ codegen can't convert more than one cxx::bridge mod
// per file, so that's why we need to put this outside of lib.rs. All of this
// could go into module.rs instead, but for now its purpose is narrowly scoped
// for testing aliasing between cxx::bridge mods, so we'll keep it that way and
// start a new mod here.

// Rustfmt mangles the extern type alias.
// https://github.com/rust-lang/rustfmt/issues/4159
#[rustfmt::skip]
#[cxx::bridge(namespace = "tests")]
pub mod ffi2 {
    impl UniquePtr<D> {}
    impl UniquePtr<E> {}
    impl UniquePtr<F> {}
    impl UniquePtr<G> {}

    extern "C" {
        include!("tests/ffi/tests.h");

        type D = crate::other::D;
        type E = crate::other::E;
        #[namespace = "F"]
        type F = crate::other::f::F;
        #[namespace = "G"]
        type G = crate::other::G;

        #[namespace = "H"]
        type H;

        fn c_take_trivial_ptr(d: UniquePtr<D>);
        fn c_take_trivial_ref(d: &D);
        fn c_take_trivial(d: D);
        fn c_take_trivial_ns_ptr(g: UniquePtr<G>);
        fn c_take_trivial_ns_ref(g: &G);
        fn c_take_trivial_ns(g: G);
        fn c_take_opaque_ptr(e: UniquePtr<E>);
        fn c_take_opaque_ref(e: &E);
        fn c_take_opaque_ns_ptr(e: UniquePtr<F>);
        fn c_take_opaque_ns_ref(e: &F);
        fn c_return_trivial_ptr() -> UniquePtr<D>;
        fn c_return_trivial() -> D;
        fn c_return_trivial_ns_ptr() -> UniquePtr<G>;
        fn c_return_trivial_ns() -> G;
        fn c_return_opaque_ptr() -> UniquePtr<E>;
        fn c_return_ns_opaque_ptr() -> UniquePtr<F>;  
        fn c_return_ns_unique_ptr() -> UniquePtr<H>;
        fn c_take_ref_ns_c(h: &H);

        #[namespace = "other"]
        fn ns_c_take_trivial(d: D);
        #[namespace = "other"]
        fn ns_c_return_trivial() -> D;

        #[namespace = "I"]
        type I;

        fn get(self: &I) -> u32;

        #[namespace = "I"]
        fn ns_c_return_unique_ptr_ns() -> UniquePtr<I>;
    }
}

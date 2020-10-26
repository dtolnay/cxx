// To test receivers on a type in a namespace outide
// the default. cxx::bridge blocks can only have a single
// receiver type, and there can only be one such block per,
// which is why this is outside.

#[rustfmt::skip]
#[cxx::bridge(namespace = tests)]
pub mod ffi3 {

    extern "C" {
        include!("tests/ffi/tests.h");

        #[namespace (namespace = I)]
        type I;

        fn get(self: &I) -> u32;

        #[namespace (namespace = I)]
        fn ns_c_return_unique_ptr_ns() -> UniquePtr<I>;
    }
}

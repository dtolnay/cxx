#[cxx::bridge(namespace = "tests")]
pub mod ffi {
    unsafe extern "C++" {
        include!("tests/ffi/tests.h");

        type C = crate::ffi::C;

        fn c_take_unique_ptr(c: UniquePtr<C>);
    }
}

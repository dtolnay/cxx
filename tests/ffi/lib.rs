#[cxx::bridge(namespace = tests)]
pub mod ffi {
    extern "C" {
        include!("tests/ffi/tests.h");
    }
}

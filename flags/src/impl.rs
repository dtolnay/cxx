#![allow(unused_assignments, unused_mut, unused_variables)]
pub const STD: &str = {
    let mut flag = "c++11";

    #[cfg(feature = "c++14")]
    (flag = "c++14");

    #[cfg(feature = "c++17")]
    (flag = "c++17");

    #[cfg(feature = "c++20")]
    (flag = "c++20");

    flag
};

pub const EXCEPTIONS: &str = {
    let mut flag = "RUST_CXX_ALLOW_EXCEPTIONS";

    #[cfg(feature = "no_exceptions")]
    (flag = "RUST_CXX_NO_EXCEPTIONS");

    flag
};

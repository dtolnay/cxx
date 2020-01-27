use cxx::{CxxString, UniquePtr};

#[cxx::bridge(namespace = tests)]
pub mod ffi {
    struct Shared {
        z: usize,
    }

    extern "C" {
        include!("tests/ffi/tests.h");

        type C;

        fn c_return_primitive() -> usize;
        fn c_return_shared() -> Shared;
        //TODO fn c_return_box() -> Box<R>;
        fn c_return_unique_ptr() -> UniquePtr<C>;
        fn c_return_ref(shared: &Shared) -> &usize;
        fn c_return_str(shared: &Shared) -> &str;
        fn c_return_rust_string() -> String;
        fn c_return_unique_ptr_string() -> UniquePtr<CxxString>;

        fn c_take_primitive(n: usize);
        fn c_take_shared(shared: Shared);
        fn c_take_box(r: Box<R>);
        fn c_take_unique_ptr(c: UniquePtr<C>);
        //TODO fn c_take_ref_r(r: &R);
        fn c_take_ref_c(c: &C);
        fn c_take_str(s: &str);
        fn c_take_rust_string(s: String);
        fn c_take_unique_ptr_string(s: UniquePtr<CxxString>);
    }

    extern "Rust" {
        type R;

        fn r_return_primitive() -> usize;
        fn r_return_shared() -> Shared;
        //TODO fn r_return_box() -> Box<R>;
        //TODO fn r_return_unique_ptr() -> UniquePtr<C>;
        fn r_return_ref(shared: &Shared) -> &usize;
        fn r_return_str(shared: &Shared) -> &str;
        fn r_return_rust_string() -> String;
        //TODO fn r_return_unique_ptr_string() -> UniquePtr<CxxString>;

        fn r_take_primitive(n: usize);
        fn r_take_shared(shared: Shared);
        fn r_take_box(r: Box<R>);
        fn r_take_unique_ptr(c: UniquePtr<C>);
        fn r_take_ref_r(r: &R);
        fn r_take_ref_c(c: &C);
        fn r_take_str(s: &str);
        fn r_take_rust_string(s: String);
        fn r_take_unique_ptr_string(s: UniquePtr<CxxString>);
    }
}

type R = ();

fn r_return_primitive() -> usize {
    2020
}

fn r_return_shared() -> ffi::Shared {
    ffi::Shared { z: 2020 }
}

fn r_return_ref(shared: &ffi::Shared) -> &usize {
    &shared.z
}

fn r_return_str(shared: &ffi::Shared) -> &str {
    let _ = shared;
    "2020"
}

fn r_return_rust_string() -> String {
    "2020".to_owned()
}

fn r_take_primitive(n: usize) {
    let _ = n;
}

fn r_take_shared(shared: ffi::Shared) {
    let _ = shared;
}

fn r_take_box(r: Box<R>) {
    let _ = r;
}

fn r_take_unique_ptr(c: UniquePtr<ffi::C>) {
    let _ = c;
}

fn r_take_ref_r(r: &R) {
    let _ = r;
}

fn r_take_ref_c(c: &ffi::C) {
    let _ = c;
}

fn r_take_str(s: &str) {
    let _ = s;
}

fn r_take_rust_string(s: String) {
    let _ = s;
}

fn r_take_unique_ptr_string(s: UniquePtr<CxxString>) {
    let _ = s;
}

#![allow(
    clippy::boxed_local,
    clippy::ptr_arg,
    clippy::trivially_copy_pass_by_ref
)]

pub mod module;

use cxx::{CxxString, UniquePtr};
use std::fmt::{self, Display};

#[cxx::bridge(namespace = tests)]
pub mod ffi {
    struct Shared {
        z: usize,
    }

    enum Enum {
        AVal,
        BVal = 2020,
        CVal,
    }

    extern "C" {
        include!("tests/ffi/tests.h");

        type C;

        fn c_return_primitive() -> usize;
        fn c_return_shared() -> Shared;
        fn c_return_box() -> Box<R>;
        fn c_return_unique_ptr() -> UniquePtr<C>;
        fn c_return_ref(shared: &Shared) -> &usize;
        fn c_return_str(shared: &Shared) -> &str;
        fn c_return_sliceu8(shared: &Shared) -> &[u8];
        fn c_return_rust_string() -> String;
        fn c_return_unique_ptr_string() -> UniquePtr<CxxString>;
        fn c_return_unique_ptr_vector_u8() -> UniquePtr<CxxVector<u8>>;
        fn c_return_unique_ptr_vector_f64() -> UniquePtr<CxxVector<f64>>;
        fn c_return_unique_ptr_vector_shared() -> UniquePtr<CxxVector<Shared>>;
        fn c_return_unique_ptr_vector_opaque() -> UniquePtr<CxxVector<C>>;
        fn c_return_ref_vector(c: &C) -> &CxxVector<u8>;
        fn c_return_rust_vec() -> Vec<u8>;
        fn c_return_ref_rust_vec(c: &C) -> &Vec<u8>;
        fn c_return_identity(_: usize) -> usize;
        fn c_return_sum(_: usize, _: usize) -> usize;
        fn c_return_enum(n: u16) -> Enum;

        fn c_take_primitive(n: usize);
        fn c_take_shared(shared: Shared);
        fn c_take_box(r: Box<R>);
        fn c_take_ref_r(r: &R);
        fn c_take_ref_c(c: &C);
        fn c_take_str(s: &str);
        fn c_take_sliceu8(s: &[u8]);
        fn c_take_rust_string(s: String);
        fn c_take_unique_ptr_string(s: UniquePtr<CxxString>);
        fn c_take_unique_ptr_vector_u8(v: UniquePtr<CxxVector<u8>>);
        fn c_take_unique_ptr_vector_f64(v: UniquePtr<CxxVector<f64>>);
        fn c_take_unique_ptr_vector_shared(v: UniquePtr<CxxVector<Shared>>);
        fn c_take_ref_vector(v: &CxxVector<u8>);
        fn c_take_rust_vec(v: Vec<u8>);
        fn c_take_rust_vec_shared(v: Vec<Shared>);
        fn c_take_rust_vec_shared_forward_iterator(v: Vec<Shared>);
        fn c_take_ref_rust_vec(v: &Vec<u8>);
        fn c_take_ref_rust_vec_copy(v: &Vec<u8>);
        fn c_take_callback(callback: fn(String) -> usize);
        fn c_take_enum(e: Enum);

        fn c_try_return_void() -> Result<()>;
        fn c_try_return_primitive() -> Result<usize>;
        fn c_fail_return_primitive() -> Result<usize>;
        fn c_try_return_box() -> Result<Box<R>>;
        fn c_try_return_ref(s: &String) -> Result<&String>;
        fn c_try_return_str(s: &str) -> Result<&str>;
        fn c_try_return_sliceu8(s: &[u8]) -> Result<&[u8]>;
        fn c_try_return_rust_string() -> Result<String>;
        fn c_try_return_unique_ptr_string() -> Result<UniquePtr<CxxString>>;
        fn c_try_return_rust_vec() -> Result<Vec<u8>>;
        fn c_try_return_ref_rust_vec(c: &C) -> Result<&Vec<u8>>;

        fn get(self: &C) -> usize;
        fn set(self: &mut C, n: usize) -> usize;
        fn get2(&self) -> usize;
        fn set2(&mut self, n: usize) -> usize;
        fn set_succeed(&mut self, n: usize) -> Result<usize>;
        fn get_fail(&mut self) -> Result<usize>;
    }

    extern "C" {
        type COwnedEnum;
    }

    #[repr(u32)]
    enum COwnedEnum {
        CVal1,
        CVal2,
    }

    extern "Rust" {
        type R;
        type R2;

        fn r_return_primitive() -> usize;
        fn r_return_shared() -> Shared;
        fn r_return_box() -> Box<R>;
        fn r_return_unique_ptr() -> UniquePtr<C>;
        fn r_return_ref(shared: &Shared) -> &usize;
        fn r_return_str(shared: &Shared) -> &str;
        fn r_return_rust_string() -> String;
        fn r_return_unique_ptr_string() -> UniquePtr<CxxString>;
        fn r_return_rust_vec() -> Vec<u8>;
        fn r_return_ref_rust_vec(shared: &Shared) -> &Vec<u8>;
        fn r_return_identity(_: usize) -> usize;
        fn r_return_sum(_: usize, _: usize) -> usize;
        fn r_return_enum(n: u32) -> Enum;

        fn r_take_primitive(n: usize);
        fn r_take_shared(shared: Shared);
        fn r_take_box(r: Box<R>);
        fn r_take_unique_ptr(c: UniquePtr<C>);
        fn r_take_ref_r(r: &R);
        fn r_take_ref_c(c: &C);
        fn r_take_str(s: &str);
        fn r_take_sliceu8(s: &[u8]);
        fn r_take_rust_string(s: String);
        fn r_take_unique_ptr_string(s: UniquePtr<CxxString>);
        fn r_take_rust_vec(v: Vec<u8>);
        fn r_take_ref_rust_vec(v: &Vec<u8>);
        fn r_take_enum(e: Enum);

        fn r_try_return_void() -> Result<()>;
        fn r_try_return_primitive() -> Result<usize>;
        fn r_fail_return_primitive() -> Result<usize>;

        fn r_return_r2(n: usize) -> Box<R2>;
        fn get(self: &R2) -> usize;
        fn set(self: &mut R2, n: usize) -> usize;
    }
}

pub type R = usize;

pub struct R2(usize);

impl R2 {
    fn get(&self) -> usize {
        self.0
    }

    fn set(&mut self, n: usize) -> usize {
        self.0 = n;
        n
    }
}

#[derive(Debug)]
struct Error;

impl std::error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("rust error")
    }
}

fn r_return_primitive() -> usize {
    2020
}

fn r_return_shared() -> ffi::Shared {
    ffi::Shared { z: 2020 }
}

fn r_return_box() -> Box<R> {
    Box::new(2020)
}

fn r_return_unique_ptr() -> UniquePtr<ffi::C> {
    extern "C" {
        fn cxx_test_suite_get_unique_ptr() -> *mut ffi::C;
    }
    unsafe { UniquePtr::from_raw(cxx_test_suite_get_unique_ptr()) }
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

fn r_return_unique_ptr_string() -> UniquePtr<CxxString> {
    extern "C" {
        fn cxx_test_suite_get_unique_ptr_string() -> *mut CxxString;
    }
    unsafe { UniquePtr::from_raw(cxx_test_suite_get_unique_ptr_string()) }
}

fn r_return_rust_vec() -> Vec<u8> {
    Vec::new()
}

fn r_return_ref_rust_vec(shared: &ffi::Shared) -> &Vec<u8> {
    let _ = shared;
    unimplemented!()
}

fn r_return_identity(n: usize) -> usize {
    n
}

fn r_return_sum(n1: usize, n2: usize) -> usize {
    n1 + n2
}

fn r_return_enum(n: u32) -> ffi::Enum {
    if n == 0 {
        ffi::Enum::AVal
    } else if n <= 2020 {
        ffi::Enum::BVal
    } else {
        ffi::Enum::CVal
    }
}

fn r_take_primitive(n: usize) {
    assert_eq!(n, 2020);
}

fn r_take_shared(shared: ffi::Shared) {
    assert_eq!(shared.z, 2020);
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
    assert_eq!(s, "2020");
}

fn r_take_rust_string(s: String) {
    assert_eq!(s, "2020");
}

fn r_take_sliceu8(s: &[u8]) {
    assert_eq!(s.len(), 5);
    assert_eq!(std::str::from_utf8(s).unwrap(), "2020\0");
}

fn r_take_unique_ptr_string(s: UniquePtr<CxxString>) {
    assert_eq!(s.as_ref().unwrap().to_str().unwrap(), "2020");
}

fn r_take_rust_vec(v: Vec<u8>) {
    let _ = v;
}

fn r_take_ref_rust_vec(v: &Vec<u8>) {
    let _ = v;
}

fn r_take_enum(e: ffi::Enum) {
    let _ = e;
}

fn r_try_return_void() -> Result<(), Error> {
    Ok(())
}

fn r_try_return_primitive() -> Result<usize, Error> {
    Ok(2020)
}

fn r_fail_return_primitive() -> Result<usize, Error> {
    Err(Error)
}

fn r_return_r2(n: usize) -> Box<R2> {
    Box::new(R2(n))
}

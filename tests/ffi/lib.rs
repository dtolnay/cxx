#![allow(
    clippy::boxed_local,
    clippy::just_underscores_and_digits,
    clippy::ptr_arg,
    clippy::trivially_copy_pass_by_ref
)]

pub mod extra;
pub mod module;

use cxx::{CxxString, CxxVector, UniquePtr};
use std::fmt::{self, Display};

mod other {
    use cxx::kind::{Opaque, Trivial};
    use cxx::{type_id, CxxString, ExternType};

    #[repr(C)]
    pub struct D {
        pub d: u64,
    }

    #[repr(C)]
    pub struct E {
        e: u64,
        e_str: CxxString,
    }

    pub mod f {
        use cxx::kind::Opaque;
        use cxx::{type_id, CxxString, ExternType};

        #[repr(C)]
        pub struct F {
            e: u64,
            e_str: CxxString,
        }

        unsafe impl ExternType for F {
            type Id = type_id!("F::F");
            type Kind = Opaque;
        }
    }

    #[repr(C)]
    pub struct G {
        pub g: u64,
    }

    unsafe impl ExternType for G {
        type Id = type_id!("G::G");
        type Kind = Trivial;
    }

    unsafe impl ExternType for D {
        type Id = type_id!("tests::D");
        type Kind = Trivial;
    }

    unsafe impl ExternType for E {
        type Id = type_id!("tests::E");
        type Kind = Opaque;
    }
}

#[cxx::bridge(namespace = "tests")]
pub mod ffi {
    #[derive(Clone)]
    struct Shared {
        z: usize,
    }

    struct SharedString {
        msg: String,
    }

    enum Enum {
        AVal,
        BVal = 2020,
        CVal,
    }

    #[namespace = "A"]
    #[derive(Clone)]
    struct AShared {
        z: usize,
    }

    #[namespace = "A"]
    enum AEnum {
        AAVal,
        ABVal = 2020,
        ACVal,
    }

    #[namespace = "A::B"]
    enum ABEnum {
        ABAVal,
        ABBVal = 2020,
        ABCVal,
    }

    #[namespace = "A::B"]
    #[derive(Clone)]
    struct ABShared {
        z: usize,
    }

    #[namespace = "first"]
    struct First {
        second: Box<Second>,
    }

    #[namespace = "second"]
    struct Second {
        i: i32,
    }

    extern "C" {
        include!("tests/ffi/tests.h");

        type C;

        fn c_return_primitive() -> usize;
        fn c_return_shared() -> Shared;
        fn c_return_box() -> Box<R>;
        fn c_return_unique_ptr() -> UniquePtr<C>;
        fn c_return_ref(shared: &Shared) -> &usize;
        fn c_return_mut(shared: &mut Shared) -> &mut usize;
        fn c_return_str(shared: &Shared) -> &str;
        fn c_return_sliceu8(shared: &Shared) -> &[u8];
        fn c_return_rust_string() -> String;
        fn c_return_unique_ptr_string() -> UniquePtr<CxxString>;
        fn c_return_unique_ptr_vector_u8() -> UniquePtr<CxxVector<u8>>;
        fn c_return_unique_ptr_vector_f64() -> UniquePtr<CxxVector<f64>>;
        fn c_return_unique_ptr_vector_string() -> UniquePtr<CxxVector<CxxString>>;
        fn c_return_unique_ptr_vector_shared() -> UniquePtr<CxxVector<Shared>>;
        fn c_return_unique_ptr_vector_opaque() -> UniquePtr<CxxVector<C>>;
        fn c_return_ref_vector(c: &C) -> &CxxVector<u8>;
        fn c_return_mut_vector(c: &mut C) -> &mut CxxVector<u8>;
        fn c_return_rust_vec() -> Vec<u8>;
        fn c_return_ref_rust_vec(c: &C) -> &Vec<u8>;
        fn c_return_mut_rust_vec(c: &mut C) -> &mut Vec<u8>;
        fn c_return_rust_vec_string() -> Vec<String>;
        fn c_return_identity(_: usize) -> usize;
        fn c_return_sum(_: usize, _: usize) -> usize;
        fn c_return_enum(n: u16) -> Enum;
        fn c_return_ns_ref(shared: &AShared) -> &usize;
        fn c_return_nested_ns_ref(shared: &ABShared) -> &usize;
        fn c_return_ns_enum(n: u16) -> AEnum;
        fn c_return_nested_ns_enum(n: u16) -> ABEnum;

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
        fn c_take_unique_ptr_vector_string(v: UniquePtr<CxxVector<CxxString>>);
        fn c_take_unique_ptr_vector_shared(v: UniquePtr<CxxVector<Shared>>);
        fn c_take_ref_vector(v: &CxxVector<u8>);
        fn c_take_rust_vec(v: Vec<u8>);
        fn c_take_rust_vec_shared(v: Vec<Shared>);
        fn c_take_rust_vec_string(v: Vec<String>);
        fn c_take_rust_vec_index(v: Vec<u8>);
        fn c_take_rust_vec_shared_index(v: Vec<Shared>);
        fn c_take_rust_vec_shared_push(v: Vec<Shared>);
        fn c_take_rust_vec_shared_forward_iterator(v: Vec<Shared>);
        fn c_take_ref_rust_vec(v: &Vec<u8>);
        fn c_take_ref_rust_vec_string(v: &Vec<String>);
        fn c_take_ref_rust_vec_index(v: &Vec<u8>);
        fn c_take_ref_rust_vec_copy(v: &Vec<u8>);
        fn c_take_ref_shared_string(s: &SharedString) -> &SharedString;
        fn c_take_callback(callback: fn(String) -> usize);
        fn c_take_enum(e: Enum);
        fn c_take_ns_enum(e: AEnum);
        fn c_take_nested_ns_enum(e: ABEnum);
        fn c_take_ns_shared(shared: AShared);
        fn c_take_nested_ns_shared(shared: ABShared);
        fn c_take_rust_vec_ns_shared(v: Vec<AShared>);
        fn c_take_rust_vec_nested_ns_shared(v: Vec<ABShared>);

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
        fn c_try_return_rust_vec_string() -> Result<Vec<String>>;
        fn c_try_return_ref_rust_vec(c: &C) -> Result<&Vec<u8>>;

        fn get(self: &C) -> usize;
        fn set(self: &mut C, n: usize) -> usize;
        fn get2(&self) -> usize;
        fn set2(&mut self, n: usize) -> usize;
        fn set_succeed(&mut self, n: usize) -> Result<usize>;
        fn get_fail(&mut self) -> Result<usize>;
        fn c_method_on_shared(self: &Shared) -> usize;

        #[rust_name = "i32_overloaded_method"]
        fn cOverloadedMethod(&self, x: i32) -> String;
        #[rust_name = "str_overloaded_method"]
        fn cOverloadedMethod(&self, x: &str) -> String;
        #[rust_name = "i32_overloaded_function"]
        fn cOverloadedFunction(x: i32) -> String;
        #[rust_name = "str_overloaded_function"]
        fn cOverloadedFunction(x: &str) -> String;

        #[namespace = "other"]
        fn ns_c_take_ns_shared(shared: AShared);
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
        fn r_return_mut(shared: &mut Shared) -> &mut usize;
        fn r_return_str(shared: &Shared) -> &str;
        fn r_return_rust_string() -> String;
        fn r_return_unique_ptr_string() -> UniquePtr<CxxString>;
        fn r_return_rust_vec() -> Vec<u8>;
        fn r_return_rust_vec_string() -> Vec<String>;
        fn r_return_ref_rust_vec(shared: &Shared) -> &Vec<u8>;
        fn r_return_mut_rust_vec(shared: &mut Shared) -> &mut Vec<u8>;
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
        fn r_take_ref_vector(v: &CxxVector<u8>);
        fn r_take_ref_empty_vector(v: &CxxVector<u64>);
        fn r_take_rust_vec(v: Vec<u8>);
        fn r_take_rust_vec_string(v: Vec<String>);
        fn r_take_ref_rust_vec(v: &Vec<u8>);
        fn r_take_ref_rust_vec_string(v: &Vec<String>);
        fn r_take_enum(e: Enum);

        fn r_try_return_void() -> Result<()>;
        fn r_try_return_primitive() -> Result<usize>;
        fn r_try_return_box() -> Result<Box<R>>;
        fn r_fail_return_primitive() -> Result<usize>;

        fn r_return_r2(n: usize) -> Box<R2>;
        fn get(self: &R2) -> usize;
        fn set(self: &mut R2, n: usize) -> usize;
        fn r_method_on_shared(self: &Shared) -> String;

        #[cxx_name = "rAliasedFunction"]
        fn r_aliased_function(x: i32) -> String;
    }

    struct Dag0 {
        i: i32,
    }

    struct Dag1 {
        dag2: Dag2,
        vec: Vec<Dag3>,
    }

    struct Dag2 {
        dag4: Dag4,
    }

    struct Dag3 {
        dag1: Dag1,
    }

    struct Dag4 {
        dag0: Dag0,
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

impl ffi::Shared {
    fn r_method_on_shared(&self) -> String {
        "2020".to_owned()
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

fn r_return_mut(shared: &mut ffi::Shared) -> &mut usize {
    &mut shared.z
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

fn r_return_rust_vec_string() -> Vec<String> {
    Vec::new()
}

fn r_return_ref_rust_vec(shared: &ffi::Shared) -> &Vec<u8> {
    let _ = shared;
    unimplemented!()
}

fn r_return_mut_rust_vec(shared: &mut ffi::Shared) -> &mut Vec<u8> {
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

fn r_take_ref_vector(v: &CxxVector<u8>) {
    let slice = v.as_slice();
    assert_eq!(slice, [20, 2, 0]);
}

fn r_take_ref_empty_vector(v: &CxxVector<u64>) {
    assert!(v.as_slice().is_empty());
    assert!(v.is_empty());
}

fn r_take_rust_vec(v: Vec<u8>) {
    let _ = v;
}

fn r_take_rust_vec_string(v: Vec<String>) {
    let _ = v;
}

fn r_take_ref_rust_vec(v: &Vec<u8>) {
    let _ = v;
}

fn r_take_ref_rust_vec_string(v: &Vec<String>) {
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

fn r_try_return_box() -> Result<Box<R>, Error> {
    Ok(Box::new(2020))
}

fn r_fail_return_primitive() -> Result<usize, Error> {
    Err(Error)
}

fn r_return_r2(n: usize) -> Box<R2> {
    Box::new(R2(n))
}

fn r_aliased_function(x: i32) -> String {
    x.to_string()
}

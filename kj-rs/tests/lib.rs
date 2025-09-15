#![allow(clippy::needless_lifetimes)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::unused_async)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::should_panic_without_expect)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::should_panic_without_expect)]
#![allow(clippy::missing_panics_doc)]

mod test_date;
mod test_futures;
mod test_maybe;
mod test_own;
mod test_refcount;

use test_futures::{
    new_awaiting_future_i32, new_error_handling_future_void_infallible, new_errored_future_void,
    new_kj_errored_future_void, new_layered_ready_future_void, new_naive_select_future_void,
    new_pending_future_void, new_ready_future_i32, new_ready_future_void,
    new_threaded_delay_future_void, new_waking_future_void, new_wrapped_waker_future_void,
};

use test_maybe::{
    take_maybe_own, take_maybe_own_ret, take_maybe_ref, take_maybe_ref_ret, take_maybe_shared,
    take_maybe_shared_ret,
};

use test_refcount::{modify_own_ret_arc, modify_own_ret_rc};

use kj_rs::KjOwn;

type Result<T> = std::io::Result<T>;
type Error = std::io::Error;

#[cxx::bridge(namespace = "kj_rs_demo")]
mod ffi {
    struct Shared {
        i: i64,
    }

    unsafe extern "C++" {
        include!("kj-rs-demo/test-promises.h");

        async fn new_ready_promise_void();
        async fn new_pending_promise_void();
        async fn new_coroutine_promise_void();

        async fn new_errored_promise_void();
        async fn new_ready_promise_i32(value: i32) -> i32;
        async fn new_ready_promise_shared_type() -> Shared;
    }

    // Helper functions to test `kj_rs::KjOwn`
    unsafe extern "C++" {
        include!("kj-rs-demo/test-own.h");
        type OpaqueCxxClass;

        #[cxx_name = "getData"]
        fn get_data(&self) -> u64;
        #[cxx_name = "setData"]
        fn set_data(self: Pin<&mut OpaqueCxxClass>, val: u64);

        #[allow(dead_code)]
        fn cxx_kj_own() -> KjOwn<OpaqueCxxClass>;
        fn null_kj_own() -> KjOwn<OpaqueCxxClass>;
        #[allow(dead_code)]
        fn give_own_back(own: KjOwn<OpaqueCxxClass>);
        #[allow(dead_code)]
        fn modify_own_return_test();
        #[allow(dead_code)]
        fn breaking_things() -> KjOwn<OpaqueCxxClass>;

        #[allow(dead_code)]
        fn own_integer() -> KjOwn<i64>;
        #[allow(dead_code)]
        fn own_integer_attached() -> KjOwn<i64>;

        #[allow(dead_code)]
        fn null_exception_test_driver_1() -> String;
        #[allow(dead_code)]
        fn null_exception_test_driver_2() -> String;
        #[allow(dead_code)]
        fn rust_take_own_driver();
    }

    unsafe extern "C++" {
        include!("kj-rs-demo/test-refcount.h");

        type OpaqueRefcountedClass;

        #[allow(dead_code)]
        fn get_rc() -> KjRc<OpaqueRefcountedClass>;
        #[allow(dead_code)]
        #[cxx_name = "getData"]
        fn get_data(&self) -> u64;
        #[allow(dead_code)]
        #[cxx_name = "setData"]
        fn set_data(self: Pin<&mut OpaqueRefcountedClass>, data: u64);

        #[allow(dead_code)]
        fn give_rc_back(rc: KjRc<OpaqueRefcountedClass>);
    }

    unsafe extern "C++" {
        include!("kj-rs-demo/test-refcount.h");

        type OpaqueAtomicRefcountedClass;

        #[allow(dead_code)]
        fn get_arc() -> KjArc<OpaqueAtomicRefcountedClass>;

        #[allow(dead_code)]
        #[cxx_name = "getData"]
        fn get_data(&self) -> u64;
        #[allow(dead_code)]
        #[cxx_name = "setData"]
        fn set_data(self: Pin<&mut OpaqueAtomicRefcountedClass>, data: u64);

        #[allow(dead_code)]
        fn give_arc_back(arc: KjArc<OpaqueAtomicRefcountedClass>);
    }

    extern "Rust" {
        fn modify_own_ret_rc(rc: KjRc<OpaqueRefcountedClass>) -> KjRc<OpaqueRefcountedClass>;
        fn modify_own_ret_arc(
            arc: KjArc<OpaqueAtomicRefcountedClass>,
        ) -> KjArc<OpaqueAtomicRefcountedClass>;
    }

    // Helper function to test moving `Own` to C++
    extern "Rust" {
        fn modify_own_return(cpp_own: KjOwn<OpaqueCxxClass>) -> KjOwn<OpaqueCxxClass>;
        fn take_own(cpp_own: KjOwn<OpaqueCxxClass>);
        fn get_null() -> KjOwn<OpaqueCxxClass>;
    }

    unsafe extern "C++" {
        include!("kj-rs-demo/test-maybe.h");

        #[allow(dead_code)]
        fn test_maybe_reference_shared_own_driver();

        #[allow(dead_code)]
        fn return_maybe() -> KjMaybe<i64>;
        #[allow(dead_code)]
        fn return_maybe_none() -> KjMaybe<i64>;
        #[allow(dead_code)]
        fn return_maybe_ref_some<'a>() -> KjMaybe<&'a i64>;
        #[allow(dead_code)]
        fn return_maybe_ref_none<'a>() -> KjMaybe<&'a i64>;
        #[allow(dead_code)]
        fn return_maybe_shared_some() -> KjMaybe<Shared>;
        #[allow(dead_code)]
        fn return_maybe_shared_none() -> KjMaybe<Shared>;
        #[allow(dead_code)]
        fn return_maybe_own_some() -> KjMaybe<KjOwn<OpaqueCxxClass>>;
        #[allow(dead_code)]
        fn return_maybe_own_none() -> KjMaybe<KjOwn<OpaqueCxxClass>>;
        #[allow(dead_code)]
        fn take_maybe_own_cxx(own: KjMaybe<KjOwn<OpaqueCxxClass>>);

        #[allow(dead_code)]
        fn cxx_take_maybe_shared_some(maybe: KjMaybe<Shared>);
        #[allow(dead_code)]
        fn cxx_take_maybe_shared_none(maybe: KjMaybe<Shared>);
        #[allow(dead_code)]
        fn cxx_take_maybe_ref_shared_some(maybe: KjMaybe<&Shared>);
        #[allow(dead_code)]
        fn cxx_take_maybe_ref_shared_none(maybe: KjMaybe<&Shared>);
    }

    unsafe extern "C++" {
        include!("kj-rs-demo/test-maybe.h");

        #[allow(dead_code)]
        fn test_maybe_u8_some() -> KjMaybe<u8>;
        #[allow(dead_code)]
        fn test_maybe_u16_some() -> KjMaybe<u16>;
        #[allow(dead_code)]
        fn test_maybe_u32_some() -> KjMaybe<u32>;
        #[allow(dead_code)]
        fn test_maybe_u64_some() -> KjMaybe<u64>;
        #[allow(dead_code)]
        fn test_maybe_usize_some() -> KjMaybe<usize>;
        #[allow(dead_code)]
        fn test_maybe_i8_some() -> KjMaybe<i8>;
        #[allow(dead_code)]
        fn test_maybe_i16_some() -> KjMaybe<i16>;
        #[allow(dead_code)]
        fn test_maybe_i32_some() -> KjMaybe<i32>;
        #[allow(dead_code)]
        fn test_maybe_i64_some() -> KjMaybe<i64>;
        #[allow(dead_code)]
        fn test_maybe_isize_some() -> KjMaybe<isize>;
        #[allow(dead_code)]
        fn test_maybe_f32_some() -> KjMaybe<f32>;
        #[allow(dead_code)]
        fn test_maybe_f64_some() -> KjMaybe<f64>;
        #[allow(dead_code)]
        fn test_maybe_bool_some() -> KjMaybe<bool>;
        #[allow(dead_code)]
        fn test_maybe_str_some() -> KjMaybe<&'static str>;
        #[allow(dead_code)]
        fn test_maybe_u8_slice_some() -> KjMaybe<&'static [u8]>;
        #[allow(dead_code)]
        fn test_maybe_u8_none() -> KjMaybe<u8>;
        #[allow(dead_code)]
        fn test_maybe_u16_none() -> KjMaybe<u16>;
        #[allow(dead_code)]
        fn test_maybe_u32_none() -> KjMaybe<u32>;
        #[allow(dead_code)]
        fn test_maybe_u64_none() -> KjMaybe<u64>;
        #[allow(dead_code)]
        fn test_maybe_usize_none() -> KjMaybe<usize>;
        #[allow(dead_code)]
        fn test_maybe_i8_none() -> KjMaybe<i8>;
        #[allow(dead_code)]
        fn test_maybe_i16_none() -> KjMaybe<i16>;
        #[allow(dead_code)]
        fn test_maybe_i32_none() -> KjMaybe<i32>;
        #[allow(dead_code)]
        fn test_maybe_i64_none() -> KjMaybe<i64>;
        #[allow(dead_code)]
        fn test_maybe_isize_none() -> KjMaybe<isize>;
        #[allow(dead_code)]
        fn test_maybe_f32_none() -> KjMaybe<f32>;
        #[allow(dead_code)]
        fn test_maybe_f64_none() -> KjMaybe<f64>;
        #[allow(dead_code)]
        fn test_maybe_bool_none() -> KjMaybe<bool>;

        #[allow(dead_code)]
        fn test_maybe_str_none() -> KjMaybe<&'static str>;
        #[allow(dead_code)]
        fn test_maybe_u8_slice_none() -> KjMaybe<&'static [u8]>;
    }

    extern "Rust" {
        fn take_maybe_own_ret(
            val: KjMaybe<KjOwn<OpaqueCxxClass>>,
        ) -> KjMaybe<KjOwn<OpaqueCxxClass>>;
        fn take_maybe_own(val: KjMaybe<KjOwn<OpaqueCxxClass>>);
        unsafe fn take_maybe_ref_ret<'a>(val: KjMaybe<&'a u64>) -> KjMaybe<&'a u64>;
        fn take_maybe_ref(val: KjMaybe<&u64>);
        fn take_maybe_shared_ret(val: KjMaybe<Shared>) -> KjMaybe<Shared>;
        fn take_maybe_shared(val: KjMaybe<Shared>);
    }

    enum CloningAction {
        None,
        CloneSameThread,
        CloneBackgroundThread,
        WakeByRefThenCloneSameThread,
    }

    enum WakingAction {
        None,
        WakeByRefSameThread,
        WakeByRefBackgroundThread,
        WakeSameThread,
        WakeBackgroundThread,
    }

    // Helper functions to create BoxFutureVoids for testing purposes.
    extern "Rust" {
        async fn new_pending_future_void();
        async fn new_ready_future_void();
        async fn new_ready_future_shared_type() -> Shared;
        async fn new_waking_future_void(cloning_action: CloningAction, waking_action: WakingAction);
        async fn new_threaded_delay_future_void();
        async fn new_layered_ready_future_void() -> Result<()>;

        async fn new_naive_select_future_void() -> Result<()>;
        async fn new_wrapped_waker_future_void() -> Result<()>;

        async fn new_errored_future_void() -> Result<()>;

        async fn new_kj_errored_future_void() -> Result<()>;

        async fn new_error_handling_future_void_infallible();

        async fn new_awaiting_future_i32() -> Result<()>;
        async fn new_ready_future_i32(value: i32) -> Result<i32>;
        async fn new_pass_through_feature_shared() -> Shared;
    }

    // these are used to check compilation only
    extern "Rust" {

        async unsafe fn lifetime_arg_void<'a>(buf: &'a [u8]);
        async unsafe fn lifetime_arg_result<'a>(buf: &'a [u8]) -> Result<()>;
    }
}

unsafe impl Send for ffi::OpaqueAtomicRefcountedClass {}
unsafe impl Sync for ffi::OpaqueAtomicRefcountedClass {}

pub fn modify_own_return(mut own: KjOwn<ffi::OpaqueCxxClass>) -> KjOwn<ffi::OpaqueCxxClass> {
    own.pin_mut().set_data(72);
    own
}

pub fn get_null() -> KjOwn<ffi::OpaqueCxxClass> {
    ffi::null_kj_own()
}

pub fn take_own(cpp_own: KjOwn<ffi::OpaqueCxxClass>) {
    assert_eq!(cpp_own.get_data(), 14);
    // The point of this function is to drop the [`Own`] from rust and this makes
    // it explicit, while avoiding a clippy lint
    std::mem::drop(cpp_own);
}

pub async fn lifetime_arg_void<'a>(_buf: &'a [u8]) {}

pub async fn lifetime_arg_result<'a>(_buf: &'a [u8]) -> Result<()> {
    Ok(())
}

/// # Panics
/// - if c++ side throws exception
pub async fn new_pass_through_feature_shared() -> ffi::Shared {
    ffi::new_ready_promise_shared_type().await.unwrap()
}

async fn new_ready_future_shared_type() -> ffi::Shared {
    ffi::Shared { i: 42 }
}

#[cfg(test)]
mod tests {
    use crate::ffi;

    #[allow(clippy::let_underscore_future)]
    #[test]
    fn compilation() {
        // these promises can't be driven by rust side, just check that everything compiles.
        let _ = ffi::new_ready_promise_void();
        let _ = ffi::new_ready_promise_i32(42);
        let _ = ffi::new_ready_promise_shared_type();
    }
}

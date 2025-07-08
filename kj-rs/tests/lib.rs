#![allow(clippy::needless_lifetimes)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::unused_async)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::should_panic_without_expect)]
#![allow(clippy::missing_panics_doc)]

mod test_futures;
mod test_own;

use test_futures::{
    new_awaiting_future_i32, new_error_handling_future_void_infallible, new_errored_future_void,
    new_layered_ready_future_void, new_naive_select_future_void, new_pending_future_void,
    new_ready_future_i32, new_ready_future_void, new_threaded_delay_future_void,
    new_waking_future_void, new_wrapped_waker_future_void,
};

use kj_rs::repr::Own;

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

    // Helper functions to test `kj_rs::Own`
    unsafe extern "C++" {
        include!("kj-rs-demo/test-own.h");
        type OpaqueCxxClass;

        #[cxx_name = "getData"]
        fn get_data(&self) -> u64;
        #[cxx_name = "setData"]
        fn set_data(self: Pin<&mut OpaqueCxxClass>, val: u64);

        #[allow(dead_code)]
        fn cxx_kj_own() -> Own<OpaqueCxxClass>;
        fn null_kj_own() -> Own<OpaqueCxxClass>;
        #[allow(dead_code)]
        fn give_own_back(own: Own<OpaqueCxxClass>);
        #[allow(dead_code)]
        fn modify_own_return_test();
        #[allow(dead_code)]
        fn breaking_things() -> Own<OpaqueCxxClass>;

        #[allow(dead_code)]
        fn own_integer() -> Own<i64>;
        #[allow(dead_code)]
        fn own_integer_attached() -> Own<i64>;

        #[allow(dead_code)]
        fn null_exception_test_driver_1() -> String;
        #[allow(dead_code)]
        fn null_exception_test_driver_2() -> String;
        #[allow(dead_code)]
        fn rust_take_own_driver();
    }

    // Helper function to test moving `Own` to C++
    extern "Rust" {
        fn modify_own_return(cpp_own: Own<OpaqueCxxClass>) -> Own<OpaqueCxxClass>;
        fn take_own(cpp_own: Own<OpaqueCxxClass>);
        fn get_null() -> Own<OpaqueCxxClass>;
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

pub fn modify_own_return(mut own: Own<ffi::OpaqueCxxClass>) -> Own<ffi::OpaqueCxxClass> {
    own.pin_mut().set_data(72);
    own
}

pub fn get_null() -> Own<ffi::OpaqueCxxClass> {
    ffi::null_kj_own()
}

pub fn take_own(cpp_own: Own<ffi::OpaqueCxxClass>) {
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

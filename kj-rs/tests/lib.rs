#![allow(clippy::needless_lifetimes)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::unused_async)]

mod test_futures;

use test_futures::{
    new_awaiting_future_i32, new_error_handling_future_void_infallible, new_errored_future_void,
    new_layered_ready_future_void, new_naive_select_future_void, new_pending_future_void,
    new_ready_future_i32, new_ready_future_void, new_threaded_delay_future_void,
    new_waking_future_void, new_wrapped_waker_future_void,
};

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

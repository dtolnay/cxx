// This file contains boilerplate which must occur once per crate, rather than once per type.

// NOTE: FuturePollStatus must be kept in sync with the C++ enum of the same name in future.h
// Ideally, this would live in kj-rs's `crate::ffi` module, and code which depends on kj-rs would be
// able to include `kj-rs/src/lib.rs.h`. I couldn't figure out how to expose that generated lib.rs.h
// header to Bazel dependents, though, so I'm just splatting it here.
#[derive(Copy, Clone, PartialEq, Eq)]
#[repr(transparent)]
pub struct FuturePollStatus {
    pub repr: u8,
}

#[allow(non_upper_case_globals)]
impl FuturePollStatus {
    pub const Pending: Self = Self { repr: 0 };
    pub const Complete: Self = Self { repr: 1 };
    pub const Error: Self = Self { repr: 2 };
}

// These types are shared with C++ code.
pub(crate) mod repr {
    use std::{
        ffi::c_void,
        pin::Pin,
        task::{Context, Poll, Waker},
    };

    use static_assertions::assert_eq_size;

    use super::FuturePollStatus;
    use crate::KjWaker;

    type PollCallback = unsafe extern "C" fn(
        fut: *mut c_void,
        waker: *const c_void,
        ret: *mut c_void,
    ) -> FuturePollStatus;

    type DropCallback = unsafe extern "C" fn(fut: *mut c_void);

    type FuturePtr<'a, T> = *mut (dyn Future<Output = Result<T, String>> + Send + 'a);

    /// Represents a `dyn Future<Output = Result<T, String>>` + Send.
    #[repr(C)]
    pub struct RustFuture<'a, T> {
        pub fut: FuturePtr<'a, T>,
        pub poll: PollCallback,
        pub drop: DropCallback,
    }

    type InfallibleFuturePtr<'a, T> = *mut (dyn Future<Output = T> + Send + 'a);

    /// Represents a `dyn Future<Output = T> + Send` where T is not a Result.
    #[repr(C)]
    pub struct RustInfallibleFuture<'a, T> {
        pub fut: InfallibleFuturePtr<'a, T>,
        pub poll: PollCallback,
        pub drop: DropCallback,
    }

    // `RustFuture<T>` and `RustInfallibleFuture<T>` have the same layout.
    // They exist separately because of rust trait type system limitations.
    assert_eq_size!(RustFuture<()>, RustInfallibleFuture<()>);

    assert_eq_size!(RustFuture<()>, [*mut c_void; 4]);
    assert_eq_size!(RustInfallibleFuture<()>, [*mut c_void; 4]);

    impl<T: Unpin> RustFuture<'_, T> {
        pub(crate) unsafe extern "C" fn poll(
            fut: *mut c_void,
            waker: *const c_void,
            ret: *mut c_void,
        ) -> FuturePollStatus {
            let fut = unsafe { *(fut.cast::<FuturePtr<T>>()) };
            let fut = unsafe { Pin::new_unchecked(&mut *fut) };
            let waker = unsafe { &*waker.cast::<KjWaker>() };
            let waker = Waker::from(waker);
            let mut context = Context::from_waker(&waker);
            match fut.poll(&mut context) {
                Poll::Ready(Ok(value)) => {
                    unsafe { std::ptr::write(ret.cast::<T>(), value) };
                    FuturePollStatus::Complete
                }
                Poll::Ready(Err(error)) => {
                    unsafe { std::ptr::write(ret.cast::<String>(), error.to_string()) };
                    FuturePollStatus::Error
                }
                Poll::Pending => FuturePollStatus::Pending,
            }
        }

        pub(crate) unsafe extern "C" fn drop_in_place(fut: *mut c_void) {
            let fut = unsafe { *(fut.cast::<FuturePtr<T>>()) };
            let fut = unsafe { Box::from_raw(fut) };
            let fut = unsafe { Pin::new_unchecked(fut) };
            drop(fut);
        }
    }

    impl<T: Unpin> RustInfallibleFuture<'_, T> {
        pub(crate) unsafe extern "C" fn poll(
            fut: *mut c_void,
            waker: *const c_void,
            ret: *mut c_void,
        ) -> FuturePollStatus {
            let fut = unsafe { *(fut.cast::<InfallibleFuturePtr<T>>()) };
            let fut = unsafe { Pin::new_unchecked(&mut *fut) };
            let waker = unsafe { &*waker.cast::<KjWaker>() };
            let waker = Waker::from(waker);
            let mut context = Context::from_waker(&waker);
            match fut.poll(&mut context) {
                Poll::Ready(value) => {
                    unsafe { std::ptr::write(ret.cast::<T>(), value) };
                    FuturePollStatus::Complete
                }
                Poll::Pending => FuturePollStatus::Pending,
            }
        }

        pub(crate) unsafe extern "C" fn drop_in_place(fut: *mut c_void) {
            let fut = unsafe { *(fut.cast::<InfallibleFuturePtr<T>>()) };
            let fut = unsafe { Box::from_raw(fut) };
            let fut = unsafe { Pin::new_unchecked(fut) };
            drop(fut);
        }
    }

    #[must_use]
    pub fn future<'a, T: Unpin>(
        fut: Pin<Box<dyn Future<Output = Result<T, String>> + Send + 'a>>,
    ) -> RustFuture<'a, T> {
        let fut = Box::into_raw(unsafe { Pin::into_inner_unchecked(fut) });
        let poll = RustFuture::<T>::poll;
        let drop = RustFuture::<T>::drop_in_place;
        RustFuture { fut, poll, drop }
    }

    #[must_use]
    pub fn infallible_future<'a, T: Unpin>(
        fut: Pin<Box<dyn Future<Output = T> + Send + 'a>>,
    ) -> RustInfallibleFuture<'a, T> {
        let fut = Box::into_raw(unsafe { Pin::into_inner_unchecked(fut) });
        let poll = RustInfallibleFuture::<T>::poll;
        let drop = RustInfallibleFuture::<T>::drop_in_place;
        RustInfallibleFuture { fut, poll, drop }
    }
}

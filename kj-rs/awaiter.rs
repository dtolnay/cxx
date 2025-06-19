use std::mem::MaybeUninit;
use std::pin::Pin;
use std::task::Context;

use crate::ffi::{RustPromiseAwaiter, RustPromiseAwaiterRepr};
use crate::waker::try_into_kj_waker_ptr;

// =======================================================================================
// Await syntax for OwnPromiseNode

use crate::OwnPromiseNode;

pub struct PromiseAwaiter<Data: std::marker::Unpin> {
    node: Option<OwnPromiseNode>,
    pub(crate) data: Data,
    awaiter: MaybeUninit<RustPromiseAwaiterRepr>,
    awaiter_initialized: bool,
    // Safety: `option_waker` must be declared after `awaiter`, because `awaiter` contains a reference
    // to `option_waker`. This ensures `option_waker` will be dropped after `awaiter`.
    option_waker: OptionWaker,
}

impl<Data: std::marker::Unpin> PromiseAwaiter<Data> {
    pub fn new(node: OwnPromiseNode, data: Data) -> Self {
        PromiseAwaiter {
            node: Some(node),
            data,
            awaiter: MaybeUninit::uninit(),
            awaiter_initialized: false,
            option_waker: OptionWaker::empty(),
        }
    }

    /// # Panics
    ///
    /// Panics if `node` is None.
    #[must_use]
    pub fn get_awaiter(mut self: Pin<&mut Self>) -> Pin<&mut RustPromiseAwaiter> {
        // Safety: We never move out of `this`.
        let this = unsafe { Pin::into_inner_unchecked(self.as_mut()) };

        // Initialize the awaiter if not already done
        if !this.awaiter_initialized {
            // On our first invocation, `node` will be Some, and `get_awaiter` will forward its
            // contents into RustPromiseAwaiter's constructor. On all subsequent invocations, `node`
            // will be None and the constructor will not run.
            let node = this.node.take();

            // Safety: `awaiter` stores `rust_waker_ptr` and uses it to call `wake()`. Note that
            // `awaiter` is `this.awaiter`, which lives before `this.option_waker`.
            // Since we drop awaiter manually, the `rust_waker_ptr` that `awaiter` stores will always
            // be valid during its lifetime.
            //
            // We pass a mutable pointer to C++. This is safe, because our use of the OptionWaker inside
            // of `std::task::Waker` is synchronized by ensuring we only allow calls to `poll()` on the
            // thread with the Promise's event loop active.
            let rust_waker_ptr = &raw mut this.option_waker;

            // Safety: The memory slot is valid and this type ensures that it will stay pinned.
            unsafe {
                crate::ffi::rust_promise_awaiter_new_in_place(
                    this.awaiter.as_mut_ptr().cast::<RustPromiseAwaiter>(),
                    rust_waker_ptr,
                    node.expect("node should be Some in call to init()"),
                );
            }
            this.awaiter_initialized = true;
        }

        // Safety: `this.awaiter` is pinned since `self` is pinned.
        unsafe {
            let raw = this.awaiter.assume_init_mut() as *mut RustPromiseAwaiterRepr;
            let raw = raw.cast::<RustPromiseAwaiter>();
            Pin::new_unchecked(&mut *raw)
        }
    }

    pub fn poll(mut self: Pin<&mut Self>, cx: &mut Context) -> bool {
        let maybe_kj_waker = try_into_kj_waker_ptr(cx.waker());
        let awaiter = self.as_mut().get_awaiter();
        // TODO(now): Safety comment.
        unsafe { awaiter.poll(&WakerRef(cx.waker()), maybe_kj_waker) }
    }
}

impl<Data: std::marker::Unpin> Drop for PromiseAwaiter<Data> {
    fn drop(&mut self) {
        if self.awaiter_initialized {
            unsafe {
                crate::ffi::rust_promise_awaiter_drop_in_place(
                    self.awaiter.as_mut_ptr().cast::<RustPromiseAwaiter>(),
                );
            }
        }
    }
}

// =======================================================================================
// OptionWaker and WakerRef

pub struct WakerRef<'a>(&'a std::task::Waker);

// This is a wrapper around `std::task::Waker`, exposed to C++. We use it in `RustPromiseAwaiter`
// to allow KJ promises to be awaited using opaque Wakers implemented in Rust.
pub struct OptionWaker {
    inner: Option<std::task::Waker>,
}

impl OptionWaker {
    pub fn empty() -> OptionWaker {
        OptionWaker { inner: None }
    }

    pub fn set(&mut self, waker: &WakerRef) {
        if let Some(w) = &mut self.inner {
            w.clone_from(waker.0);
        } else {
            self.inner = Some(waker.0.clone());
        }
    }

    pub fn set_none(&mut self) {
        self.inner = None;
    }

    pub fn wake_mut(&mut self) {
        self.inner
            .take()
            .expect(
                "OptionWaker::set() should be called before RustPromiseAwaiter::poll(); \
                OptionWaker::wake() should be called at most once after poll()",
            )
            .wake();
    }
}

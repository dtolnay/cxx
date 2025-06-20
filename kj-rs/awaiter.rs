use std::pin::Pin;

use std::task::Context;

use cxx::ExternType;

use crate::waker::try_into_kj_waker_ptr;

use crate::lazy_pin_init::LazyPinInit;

// =======================================================================================
// GuardedRustPromiseAwaiter

#[path = "awaiter.h.rs"]
mod awaiter_h;
pub use awaiter_h::GuardedRustPromiseAwaiter;

// Safety: KJ Promises are not associated with threads, but with event loops at construction time.
// Therefore, they can be polled from any thread, as long as that thread has the correct event loop
// active at the time of the call to `poll()`. If the correct event loop is not active,
// GuardedRustPromiseAwaiter's API will panic. (The Guarded- prefix refers to the C++ class template
// ExecutorGuarded, which enforces the correct event loop requirement.)
unsafe impl Send for GuardedRustPromiseAwaiter {}

impl Drop for GuardedRustPromiseAwaiter {
    fn drop(&mut self) {
        // Pin safety:
        // The pin crate suggests implementing drop traits for address-sensitive types with an inner
        // function which accepts a `Pin<&mut Type>` parameter, to help uphold pinning guarantees.
        // However, since our drop function is actually a C++ destructor to which we must pass a raw
        // pointer, there is no benefit in creating a Pin from `self`.
        //
        // https://doc.rust-lang.org/std/pin/index.html#implementing-drop-for-types-with-address-sensitive-states
        //
        // Pointer safety:
        // 1. Pointer to self is non-null, and obviously points to valid memory.
        // 2. We do not read or write to the OwnPromiseNode's memory, so there are no atomicity nor
        //    interleaved pointer/reference access concerns.
        //
        // https://doc.rust-lang.org/std/ptr/index.html#safety
        unsafe {
            crate::ffi::guarded_rust_promise_awaiter_drop_in_place(PtrGuardedRustPromiseAwaiter(
                self,
            ));
        }
    }
}

// Safety: We have a static_assert in awaiter.c++ which breaks if you change the size or alignment
// of the C++ definition of GuardedRustPromiseAwaiter, with instructions to regenerate the bindgen-
// generated type. I couldn't figure out how to static_assert on the actual generated Rust struct,
// though, so it's not perfect. Ideally we'd run bindgen in the build system.
//
// https://docs.rs/cxx/latest/cxx/trait.ExternType.html#integrating-with-bindgen-generated-types
unsafe impl ExternType for GuardedRustPromiseAwaiter {
    type Id = cxx::type_id!("::kj_rs::GuardedRustPromiseAwaiter");
    type Kind = cxx::kind::Opaque;
}

// This Ptr- type is copied from dtolnay's `Box<dyn Trait>` example:
// https://github.com/dtolnay/cxx/pull/672/files#diff-0ca4eb15f64c7f6bd562b9efb702a1f6020ee83a6f1c686054cfd3fb314edfe1R16
//
// I don't entirely understand why it is necessary. When I tried to replace it with raw pointers, I
// encountered segfaults, so I put it back.
//
// TODO(someday): Examine the generated code with and without this wrapper type and figure out why
//   it's necessary. Do we need this sort of wrapper for our macro-generated Future and Promise
//   types, too?
#[repr(transparent)]
pub struct PtrGuardedRustPromiseAwaiter(*mut GuardedRustPromiseAwaiter);

// Safety: Raw pointers are the same size in both languages.
unsafe impl ExternType for PtrGuardedRustPromiseAwaiter {
    type Id = cxx::type_id!("::kj_rs::PtrGuardedRustPromiseAwaiter");
    type Kind = cxx::kind::Trivial;
}

// =======================================================================================
// Await syntax for OwnPromiseNode

use crate::OwnPromiseNode;

pub struct PromiseAwaiter<Data: std::marker::Unpin> {
    node: Option<OwnPromiseNode>,
    pub(crate) data: Data,
    awaiter: LazyPinInit<GuardedRustPromiseAwaiter>,
    // Safety: `option_waker` must be declared after `awaiter`, because `awaiter` contains a reference
    // to `option_waker`. This ensures `option_waker` will be dropped after `awaiter`.
    option_waker: OptionWaker,
}

impl<Data: std::marker::Unpin> PromiseAwaiter<Data> {
    pub fn new(node: OwnPromiseNode, data: Data) -> Self {
        PromiseAwaiter {
            node: Some(node),
            data,
            awaiter: LazyPinInit::uninit(),
            option_waker: OptionWaker::empty(),
        }
    }

    /// # Panics
    ///
    /// Panics if `node` is None.
    #[must_use]
    pub fn get_awaiter(mut self: Pin<&mut Self>) -> Pin<&mut GuardedRustPromiseAwaiter> {
        // On our first invocation, `node` will be Some, and `get_awaiter` will forward its
        // contents into GuardedRustPromiseAwaiter's constructor. On all subsequent invocations, `node`
        // will be None and the constructor will not run.
        let node = self.as_mut().node.take();

        // Safety: `awaiter` stores `rust_waker_ptr` and uses it to call `wake()`. Note that
        // `awaiter` is `self.awaiter`, which lives before `self.option_waker`. Since struct members
        // are dropped in declaration order, the `rust_waker_ptr` that `awaiter` stores will always
        // be valid during its lifetime.
        //
        // We pass a mutable pointer to C++. This is safe, because our use of the OptionWaker inside
        // of `std::task::Waker` is synchronized by ensuring we only allow calls to `poll()` on the
        // thread with the Promise's event loop active.
        let rust_waker_ptr = &raw mut self.as_mut().option_waker;

        // Safety:
        // 1. We do not implement Unpin for PromiseAwaiter.
        // 2. Our Drop trait implementation does not move the awaiter value, nor do we use
        //    `repr(packed)` anywhere.
        // 3. The backing memory is inside our pinned Future, so we can be assured our Drop trait
        //    implementation will run before Rust re-uses the memory.
        //
        // https://doc.rust-lang.org/std/pin/index.html#choosing-pinning-to-be-structural-for-field
        let awaiter = unsafe { self.map_unchecked_mut(|s| &mut s.awaiter) };

        // Safety:
        // 1. We trust that LazyPinInit's implementation passed us a valid pointer to an
        //    uninitialized GuardedRustPromiseAwaiter.
        // 2. We do not read or write to the GuardedRustPromiseAwaiter's memory, so there are no atomicity
        //    nor interleaved pointer reference access concerns.
        //
        // https://doc.rust-lang.org/std/ptr/index.html#safety
        awaiter.get_or_init(move |ptr: *mut GuardedRustPromiseAwaiter| unsafe {
            crate::ffi::guarded_rust_promise_awaiter_new_in_place(
                PtrGuardedRustPromiseAwaiter(ptr),
                rust_waker_ptr,
                node.expect("node should be Some in call to init()"),
            );
        })
    }

    pub fn poll(mut self: Pin<&mut Self>, cx: &mut Context) -> bool {
        let maybe_kj_waker = try_into_kj_waker_ptr(cx.waker());
        let awaiter = self.as_mut().get_awaiter();
        // TODO(now): Safety comment.
        unsafe { awaiter.poll(&WakerRef(cx.waker()), maybe_kj_waker) }
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

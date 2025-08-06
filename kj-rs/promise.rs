use ::cxx::core::mem::MaybeUninit;
use cxx::ExternType;

use crate::PromiseAwaiter;

use std::marker::PhantomData;

use std::ffi::c_void;
use std::future::Future;
use std::mem::ManuallyDrop;
use std::pin::Pin;
use std::task::Context;
use std::task::Poll;

type CxxResult<T> = std::result::Result<T, cxx::KjException>;

// The inner pointer is never read on Rust's side, so Rust thinks it's dead code.
#[allow(dead_code)]
#[repr(transparent)]
pub struct OwnPromiseNode(*mut c_void /* kj::_::PromiseNode* */);

// Note: drop is not the only way for OwnPromiseNode to be destroyed.
// It is forgotten using `MaybeUninit` and its ownership passed over to c++ in `unwrap`.
impl Drop for OwnPromiseNode {
    fn drop(&mut self) {
        // Safety:
        // 1. Pointer to self is non-null, and obviously points to valid memory.
        // 2. We do not read or write to the OwnPromiseNode's memory, so there are no atomicity nor
        //    interleaved pointer/reference access concerns.
        //
        // https://doc.rust-lang.org/std/ptr/index.html#safety
        unsafe {
            crate::ffi::own_promise_node_drop_in_place(self);
        }
    }
}

// Safety: We have a static_assert in promise.c++ which breaks if you change the size or alignment
// of the C++ definition of OwnPromiseNode, with a comment directing the reader to adjust the
// OwnPromiseNode definition in this .rs file.
//
// https://docs.rs/cxx/latest/cxx/trait.ExternType.html#integrating-with-bindgen-generated-types
unsafe impl ExternType for OwnPromiseNode {
    type Id = cxx::type_id!("::kj_rs::OwnPromiseNode");
    type Kind = cxx::kind::Trivial;
}

pub trait KjPromise: Sized {
    type Output;
    type Data: std::marker::Unpin;
    fn into_own_promise_node(self) -> OwnPromiseNode;

    /// # Errors
    ///
    /// Returns an error when C++ side generated an exception.
    ///
    /// # Safety
    ///
    /// You must guarantee that `node` was previously returned from this same type's
    /// `into_own_promise_node()` implementation.
    /// node is supposed to be already resolved
    unsafe fn unwrap(node: OwnPromiseNode, data: &Self::Data) -> CxxResult<Self::Output>;
}

pub struct PromiseFuture<P: KjPromise> {
    awaiter: PromiseAwaiter<P::Data>,
    _marker: PhantomData<P>,
}

impl<P: KjPromise> PromiseFuture<P> {
    pub fn new(promise: P, data: P::Data) -> Self {
        PromiseFuture {
            awaiter: PromiseAwaiter::new(promise.into_own_promise_node(), data),
            _marker: PhantomData,
        }
    }
}

impl<P: KjPromise> Future for PromiseFuture<P> {
    type Output = CxxResult<P::Output>;
    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        // TODO(now): Safety comment.
        let mut awaiter = unsafe { self.map_unchecked_mut(|s| &mut s.awaiter) };
        if awaiter.as_mut().poll(cx) {
            let node = awaiter.as_mut().get_awaiter().take_own_promise_node();
            // TODO(now): Safety comment.
            let value = unsafe { P::unwrap(node, &awaiter.data) };
            Poll::Ready(value)
        } else {
            Poll::Pending
        }
    }
}

type UnwrapCallback =
    unsafe extern "C" fn(node: *mut c_void, ret: *mut c_void) -> cxx::private::Result;

#[repr(C)]
pub struct KjPromiseNodeImpl {
    pub node: *mut c_void,
    pub unwrap: UnwrapCallback,
}

#[allow(clippy::needless_pass_by_value)]
pub fn new_callbacks_promise_future<T>(
    r#impl: KjPromiseNodeImpl,
) -> impl Future<Output = CxxResult<T>> {
    PromiseFuture::new(
        CallbacksFuture {
            node: r#impl.node,
            _phantom: PhantomData,
        },
        FutureCallbacks {
            unwrap: r#impl.unwrap,
        },
    )
}

pub struct FutureCallbacks {
    pub unwrap: UnwrapCallback,
}

pub struct CallbacksFuture<T> {
    pub node: *mut c_void,
    _phantom: PhantomData<T>,
}

impl<T> KjPromise for CallbacksFuture<T> {
    type Output = T;
    type Data = FutureCallbacks;

    fn into_own_promise_node(self) -> OwnPromiseNode {
        OwnPromiseNode(self.node)
    }

    unsafe fn unwrap(node: OwnPromiseNode, callbacks: &FutureCallbacks) -> CxxResult<Self::Output> {
        let mut ret = MaybeUninit::<Self::Output>::uninit();
        // unwrap will take over node ownership
        let node = ManuallyDrop::new(node);

        unsafe { (callbacks.unwrap)(node.0, ret.as_mut_ptr().cast::<c_void>()).into_result() }?;
        Ok(unsafe { ret.assume_init() })
    }
}

unsafe impl<T: Send> Send for CallbacksFuture<T> {}

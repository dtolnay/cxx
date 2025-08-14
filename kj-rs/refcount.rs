//! Module for both [`KjRc`] and [`KjArc`], since they're nearly identical types

// Allows using `Refcounted` and `AtomicRefcounted` from `kj_rs::refcounted` as
// if it was a Rust trait representing a refcounted object.
pub use repr::{AtomicRefcounted, Refcounted};

pub mod repr {
    use crate::KjOwn;
    use std::ops::Deref;

    /// # Safety
    /// Should only be automatically implemented by the bridge macro
    pub unsafe trait Refcounted {
        fn is_shared(&self) -> bool;
        /// # Safety
        /// Do not call this function, instead, clone the [`KjRc`].
        unsafe fn add_ref(rc: &KjRc<Self>) -> KjRc<Self>;
    }

    /// # Safety
    /// Should only be automatically implemented by the bridge macro
    pub unsafe trait AtomicRefcounted {
        fn is_shared(&self) -> bool;
        /// # Safety
        /// Do not call this function, instead, clone the [`KjArc`].
        unsafe fn add_ref(arc: &KjArc<Self>) -> KjArc<Self>;
    }

    /// Bindings to the kj type `kj::Rc`. Represents and owned and reference counted type,
    /// like Rust's [`std::rc::Rc`].
    #[repr(C)]
    pub struct KjRc<T: Refcounted + ?Sized> {
        own: KjOwn<T>,
    }

    /// Bindings to the kj type `kj::Arc`. Represents and owned and atomically reference
    /// counted type, like Rust's [`std::sync::Arc`].
    #[repr(C)]
    pub struct KjArc<T: AtomicRefcounted + ?Sized> {
        own: KjOwn<T>,
    }

    unsafe impl<T: AtomicRefcounted> Send for KjArc<T> where T: Send {}
    unsafe impl<T: AtomicRefcounted> Sync for KjArc<T> where T: Sync {}

    impl<T: Refcounted> KjRc<T> {
        #[must_use]
        pub fn get(&self) -> *const T {
            self.own.as_ptr()
        }

        // The return value here represents exclusive access to the internal `Own`.
        // This allows for exclusive mutation of the inner value.
        pub fn get_mut(&mut self) -> Option<&mut KjOwn<T>> {
            if self.own.is_shared() {
                None
            } else {
                Some(&mut self.own)
            }
        }
    }

    impl<T: AtomicRefcounted> KjArc<T> {
        #[must_use]
        pub fn get(&self) -> *const T {
            self.own.as_ptr()
        }

        // The return value here represents exclusive access to the internal `Own`.
        // This allows for exclusive mutation of the inner value.
        pub fn get_mut(&mut self) -> Option<&mut KjOwn<T>> {
            if self.own.is_shared() {
                None
            } else {
                Some(&mut self.own)
            }
        }
    }

    impl<T: Refcounted> Deref for KjRc<T> {
        type Target = T;

        fn deref(&self) -> &Self::Target {
            &self.own
        }
    }

    impl<T: AtomicRefcounted> Deref for KjArc<T> {
        type Target = T;

        fn deref(&self) -> &Self::Target {
            &self.own
        }
    }

    /// Using clone to create another count, like how Rust does it.
    impl<T: Refcounted> Clone for KjRc<T> {
        fn clone(&self) -> Self {
            unsafe { T::add_ref(self) }
        }
    }

    impl<T: AtomicRefcounted> Clone for KjArc<T> {
        fn clone(&self) -> Self {
            unsafe { T::add_ref(self) }
        }
    }

    // No `Drop` needs to be implemented for `KjRc` or `KjArc`, because the
    // internal `Own` `Drop` is sufficient.
}

//! Module for both [`KjRc`] and [`KjArc`], since they're nearly identical types

// Allows using `AtomicRefcounted` from `kj_rs::refcounted` as a Rust trait representing an
// atomically-refcounted object.
pub use repr::AtomicRefcounted;

pub mod repr {
    use crate::KjOwn;
    use std::ffi::c_void;
    use std::ops::Deref;
    use std::pin::Pin;
    use std::ptr::NonNull;

    /// # Safety
    /// Should only be automatically implemented by the bridge macro
    pub unsafe trait AtomicRefcounted {
        fn is_shared(&self) -> bool;
        /// # Safety
        /// Do not call this function, instead, clone the [`KjArc`].
        unsafe fn add_ref(arc: &KjArc<Self>) -> KjArc<Self>;
    }

    /// Bindings to the kj type `kj::Rc`. Represents an owned and reference counted type,
    /// like Rust's [`std::rc::Rc`]. The pointee does not need to inherit `kj::Refcounted`.
    #[repr(C)]
    pub struct KjRc<T> {
        refcounted: *mut c_void,
        ptr: NonNull<T>,
    }

    /// Bindings to the kj type `kj::Arc`. Represents and owned and atomically reference
    /// counted type, like Rust's [`std::sync::Arc`].
    #[repr(C)]
    pub struct KjArc<T: AtomicRefcounted + ?Sized> {
        own: KjOwn<T>,
    }

    unsafe impl<T: AtomicRefcounted> Send for KjArc<T> where T: Send {}
    unsafe impl<T: AtomicRefcounted> Sync for KjArc<T> where T: Sync {}

    impl<T> KjRc<T> {
        #[must_use]
        pub fn is_shared(&self) -> bool {
            unsafe extern "C" {
                #[link_name = "cxxbridge$kjrs$rc$is_shared"]
                fn __is_shared(this: *const c_void) -> bool;
            }

            unsafe { __is_shared(std::ptr::from_ref(self).cast::<c_void>()) }
        }

        #[must_use]
        pub fn get(&self) -> *const T {
            self.ptr.as_ptr().cast_const()
        }

        // The return value here represents exclusive access to the pointee.
        // This allows for exclusive mutation of the inner value.
        pub fn get_mut(&mut self) -> Option<Pin<&mut T>> {
            if self.is_shared() {
                None
            } else {
                // Safety: moving the `KjRc` does not move the pointee, `is_shared()` proves that
                // this is the only active `KjRc` reference to it.
                unsafe { Some(Pin::new_unchecked(self.ptr.as_mut())) }
            }
        }
    }

    impl<T> Drop for KjRc<T> {
        fn drop(&mut self) {
            unsafe extern "C" {
                #[link_name = "cxxbridge$kjrs$rc$drop"]
                fn __drop(this: *mut c_void);
            }

            unsafe {
                __drop(std::ptr::from_mut(self).cast::<c_void>());
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

    impl<T> Deref for KjRc<T> {
        type Target = T;

        fn deref(&self) -> &Self::Target {
            // Safety: `KjRc` does not allow null pointees to cross into Rust.
            unsafe { self.ptr.as_ref() }
        }
    }

    impl<T: AtomicRefcounted> Deref for KjArc<T> {
        type Target = T;

        fn deref(&self) -> &Self::Target {
            &self.own
        }
    }

    /// Using clone to create another count, like how Rust does it.
    impl<T> Clone for KjRc<T> {
        fn clone(&self) -> Self {
            unsafe extern "C" {
                #[link_name = "cxxbridge$kjrs$rc$clone"]
                fn __clone(this: *const c_void, out: *mut c_void);
            }

            let mut ret = std::mem::MaybeUninit::<Self>::uninit();
            unsafe {
                __clone(
                    std::ptr::from_ref(self).cast::<c_void>(),
                    ret.as_mut_ptr().cast::<c_void>(),
                );
                ret.assume_init()
            }
        }
    }

    impl<T: AtomicRefcounted> Clone for KjArc<T> {
        fn clone(&self) -> Self {
            unsafe { T::add_ref(self) }
        }
    }

    // No `Drop` needs to be implemented for `KjArc`, because the internal
    // `Own` `Drop` is sufficient.
}

//! The `workerd-cxx` module containing the [`Own<T>`] type, which is bindings to the `kj::Own<T>` C++ type

use static_assertions::{assert_eq_align, assert_eq_size};
use std::{fmt, marker::PhantomData};

assert_eq_size!(repr::Own<()>, [*const (); 2]);
assert_eq_align!(repr::Own<()>, *const ());

/// When we want to use an `Own`, we want the guarantee of being not null only
/// in direct `Own<T>`, not Maybe<Own<T>>, and using a [`NonNull`] in `Own`
/// but allowing Nulls for Niche Value Optimization is undefined behavior.
#[repr(transparent)]
pub(crate) struct NonNullExceptMaybe<T>(pub(crate) *mut T, PhantomData<T>);

impl<T> NonNullExceptMaybe<T> {
    pub fn as_ptr(&self) -> *const T {
        self.0.cast()
    }

    pub unsafe fn as_ref(&self) -> &T {
        // Safety:
        //     This value will only be null when in a [`Maybe<T>`], which does niche value optimization
        //     for a null pointer, so the inner [`Own<T>`] can never be accessed if it is null
        unsafe { self.0.as_ref().unwrap() }
    }

    pub unsafe fn as_mut(&mut self) -> &mut T {
        // Safety:
        //     This value will only be null when in a [`Maybe<T>`], which does niche value optimization
        //     for a null pointer, so the inner [`Own<T>`] can never be accessed if it is null
        unsafe { self.0.as_mut().unwrap() }
    }
}

impl<T> fmt::Pointer for NonNullExceptMaybe<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Pointer::fmt(&self.0, f)
    }
}

pub mod repr {
    use super::NonNullExceptMaybe;
    use std::ffi::c_void;
    use std::fmt::{self, Debug, Display};
    use std::hash::{Hash, Hasher};
    use std::ops::Deref;
    use std::ops::DerefMut;
    use std::pin::Pin;

    /// A [`Own<T>`] represents the `kj::Own<T>`. It is a smart pointer to an opaque C++ type.
    /// Safety:
    /// - Passing a null `kj::Own` to rust is considered unsafe from the C++ side,
    ///   and it is required that this invariant is upheld in C++ code.
    /// - Currently, it is runtime asserted in the bridge macro that no null Own can be passed
    ///   to Rust
    #[repr(C)]
    pub struct Own<T> {
        pub(crate) disposer: *const c_void,
        pub(crate) ptr: NonNullExceptMaybe<T>,
    }

    /// Public-facing Own api
    impl<T> Own<T> {
        /// Returns a mutable pinned reference to the object owned by this [`Own`]
        /// if any, otherwise None.
        pub fn as_mut(&mut self) -> Pin<&mut T> {
            // Safety: Passing a null kj::Own to Rust from C++ is not supported.
            unsafe {
                let mut_reference = self.ptr.as_mut();
                Pin::new_unchecked(mut_reference)
            }
        }

        /// Returns a mutable pinned reference to the object owned by this
        /// [`Own`].
        ///
        /// ```compile_fail
        /// let mut own = ffi::cxx_kj_own();
        /// let pin1 = own.pin_mut();
        /// let pin2 = own.pin_mut();
        /// pin1.set_data(12); // Causes a compile fail, because we invalidated the first borrow
        /// ```
        ///
        /// ```compile_fail
        ///
        /// let mut own = ffi::cxx_kj_own();
        /// let pin = own.pin_mut();
        /// let moved  = own;
        /// own.set_data(143); // Compile fail, because we tried using a moved object
        /// ```
        pub fn pin_mut(&mut self) -> Pin<&mut T> {
            self.as_mut()
        }

        /// Returns a raw const pointer to the object owned by this [`Own`]
        #[must_use]
        pub fn as_ptr(&self) -> *const T {
            self.ptr.as_ptr().cast()
        }
    }

    impl<T> AsRef<T> for Own<T> {
        /// Returns a reference to the object owned by this [`Own`] if any,
        /// otherwise None.
        fn as_ref(&self) -> &T {
            // Safety: Passing a null kj::Own to Rust from C++ is not supported.
            unsafe { self.ptr.as_ref() }
        }
    }

    unsafe impl<T> Send for Own<T> where T: Send {}

    unsafe impl<T> Sync for Own<T> where T: Sync {}

    impl<T> Deref for Own<T> {
        type Target = T;

        fn deref(&self) -> &Self::Target {
            self.as_ref()
        }
    }

    impl<T> DerefMut for Own<T>
    where
        T: Unpin,
    {
        fn deref_mut(&mut self) -> &mut Self::Target {
            Pin::into_inner(self.as_mut())
        }
    }

    // Own<T> is safe to implement Unpin because moving the Own doesn't move the pointee, and
    // the drop implentation doesn't depend on the Own's location, because it's handed by virtual dispatch
    impl<T> Unpin for Own<T> {}

    impl<T> Debug for Own<T> {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "Own(ptr: {:p}, disposer: {:p})", self.ptr, self.disposer)
        }
    }

    impl<T> Display for Own<T>
    where
        T: Display,
    {
        fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            Display::fmt(self.as_ref(), formatter)
        }
    }

    impl<T> PartialEq for Own<T>
    where
        T: PartialEq,
    {
        fn eq(&self, other: &Self) -> bool {
            self.as_ref() == other.as_ref()
        }
    }

    impl<T> Eq for Own<T> where T: Eq {}

    impl<T> PartialOrd for Own<T>
    where
        T: PartialOrd,
    {
        fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
            PartialOrd::partial_cmp(&self.as_ref(), &other.as_ref())
        }
    }

    impl<T> Ord for Own<T>
    where
        T: Ord,
    {
        fn cmp(&self, other: &Self) -> std::cmp::Ordering {
            Ord::cmp(&self.as_ref(), &other.as_ref())
        }
    }

    impl<T> Hash for Own<T>
    where
        T: Hash,
    {
        fn hash<H: Hasher>(&self, state: &mut H) {
            self.as_ref().hash(state);
        }
    }

    impl<T> Drop for Own<T> {
        fn drop(&mut self) {
            unsafe extern "C" {
                #[link_name = "cxxbridge$kjrs$own$drop"]
                fn __drop(this: *mut c_void);
            }

            let this = std::ptr::from_mut::<Self>(self).cast::<c_void>();
            unsafe {
                __drop(this);
            }
        }
    }
}

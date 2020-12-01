use crate::kind::Trivial;
use crate::ExternType;
use core::ffi::c_void;
use core::fmt::{self, Debug, Display};
use core::marker::PhantomData;
use core::mem::MaybeUninit;
use core::ops::{Deref, DerefMut};
use core::pin::Pin;

/// BInding to C++ `std::shared_ptr<T>`.
#[repr(C)]
pub struct SharedPtr<T>
where
    T: SharedPtrTarget,
{
    repr: [*mut c_void; 2],
    ty: PhantomData<T>,
}

impl<T> SharedPtr<T>
where
    T: SharedPtrTarget,
{
    /// Makes a new SharedPtr wrapping a null pointer.
    ///
    /// Matches the behavior of default-constructing a std::shared\_ptr.
    pub fn null() -> Self {
        let mut shared_ptr = MaybeUninit::<SharedPtr<T>>::uninit();
        let new = shared_ptr.as_mut_ptr().cast();
        unsafe {
            T::__null(new);
            shared_ptr.assume_init()
        }
    }

    /// Allocates memory on the heap and makes a SharedPtr owner for it.
    pub fn new(value: T) -> Self
    where
        T: ExternType<Kind = Trivial>,
    {
        let mut shared_ptr = MaybeUninit::<SharedPtr<T>>::uninit();
        let new = shared_ptr.as_mut_ptr().cast();
        unsafe {
            T::__new(value, new);
            shared_ptr.assume_init()
        }
    }

    /// Chacks whether the SharedPtr does not own an object.
    ///
    /// This is the opposite of [std::shared_ptr\<T\>::operator bool](https://en.cppreference.com/w/cpp/memory/shared_ptr/operator_bool).
    pub fn is_null(&self) -> bool {
        let this = self as *const Self as *const c_void;
        let ptr = unsafe { T::__get(this) };
        ptr.is_null()
    }

    /// Returns a reference to the object owned by this SharedPtr if any,
    /// otherwise None.
    pub fn as_ref(&self) -> Option<&T> {
        let this = self as *const Self as *const c_void;
        unsafe { T::__get(this).as_ref() }
    }

    /// Returns a mutable pinned reference to the object owned by this SharedPtr
    /// if any, otherwise None.
    pub fn as_mut(&mut self) -> Option<Pin<&mut T>> {
        let this = self as *mut Self as *mut c_void;
        unsafe {
            let mut_reference = (T::__get(this) as *mut T).as_mut()?;
            Some(Pin::new_unchecked(mut_reference))
        }
    }

    /// Returns a mutable pinned reference to the object owned by this
    /// SharedPtr.
    ///
    /// # Panics
    ///
    /// Panics if the SharedPtr holds a null pointer.
    pub fn pin_mut(&mut self) -> Pin<&mut T> {
        match self.as_mut() {
            Some(target) => target,
            None => panic!("called pin_mut on a null SharedPtr<{}>", T::__NAME),
        }
    }
}

unsafe impl<T> Send for SharedPtr<T> where T: Send + Sync + SharedPtrTarget {}
unsafe impl<T> Sync for SharedPtr<T> where T: Send + Sync + SharedPtrTarget {}

impl<T> Clone for SharedPtr<T>
where
    T: SharedPtrTarget,
{
    fn clone(&self) -> Self {
        let mut shared_ptr = MaybeUninit::<SharedPtr<T>>::uninit();
        let new = shared_ptr.as_mut_ptr().cast();
        let this = self as *const Self as *mut c_void;
        unsafe {
            T::__clone(this, new);
            shared_ptr.assume_init()
        }
    }
}

impl<T> Drop for SharedPtr<T>
where
    T: SharedPtrTarget,
{
    fn drop(&mut self) {
        let this = self as *mut Self as *mut c_void;
        unsafe { T::__drop(this) }
    }
}

impl<T> Deref for SharedPtr<T>
where
    T: SharedPtrTarget,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        match self.as_ref() {
            Some(target) => target,
            None => panic!("called deref on a null SharedPtr<{}>", T::__NAME),
        }
    }
}

impl<T> DerefMut for SharedPtr<T>
where
    T: SharedPtrTarget + Unpin,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self.as_mut() {
            Some(target) => Pin::into_inner(target),
            None => panic!("called deref_mut on a null SharedPtr<{}>", T::__NAME),
        }
    }
}

impl<T> Debug for SharedPtr<T>
where
    T: Debug + SharedPtrTarget,
{
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self.as_ref() {
            None => formatter.write_str("nullptr"),
            Some(value) => Debug::fmt(value, formatter),
        }
    }
}

impl<T> Display for SharedPtr<T>
where
    T: Display + SharedPtrTarget,
{
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self.as_ref() {
            None => formatter.write_str("nullptr"),
            Some(value) => Display::fmt(value, formatter),
        }
    }
}

// Methods are private; not intended to be implemented outside of cxxbridge
// codebase.
pub unsafe trait SharedPtrTarget {
    #[doc(hidden)]
    const __NAME: &'static dyn Display;
    #[doc(hidden)]
    unsafe fn __null(new: *mut c_void);
    #[doc(hidden)]
    unsafe fn __new(value: Self, new: *mut c_void)
    where
        Self: Sized,
    {
        // Opoaque C types do not get this method because they can never exist
        // by value on the Rust side of the bridge.
        let _ = value;
        let _ = new;
        unreachable!()
    }
    #[doc(hidden)]
    unsafe fn __clone(this: *const c_void, new: *mut c_void);
    #[doc(hidden)]
    unsafe fn __get(this: *const c_void) -> *const Self;
    #[doc(hidden)]
    unsafe fn __drop(this: *mut c_void);
}

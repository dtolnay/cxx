use crate::cxx_string::CxxString;
use crate::cxx_vector::{self, CxxVector, VectorElement};
use std::ffi::c_void;
use std::fmt::{self, Debug, Display};
use std::marker::PhantomData;
use std::mem;
use std::ops::{Deref, DerefMut};
use std::ptr;

/// Binding to C++ `std::unique_ptr<T, std::default_delete<T>>`.
#[repr(C)]
pub struct UniquePtr<T>
where
    T: UniquePtrTarget,
{
    repr: *mut c_void,
    ty: PhantomData<T>,
}

impl<T> UniquePtr<T>
where
    T: UniquePtrTarget,
{
    /// Makes a new UniquePtr wrapping a null pointer.
    ///
    /// Matches the behavior of default-constructing a std::unique\_ptr.
    pub fn null() -> Self {
        UniquePtr {
            repr: T::__null(),
            ty: PhantomData,
        }
    }

    /// Allocates memory on the heap and makes a UniquePtr pointing to it.
    pub fn new(value: T) -> Self {
        UniquePtr {
            repr: T::__new(value),
            ty: PhantomData,
        }
    }

    /// Checks whether the UniquePtr does not own an object.
    ///
    /// This is the opposite of [std::unique_ptr\<T\>::operator bool](https://en.cppreference.com/w/cpp/memory/unique_ptr/operator_bool).
    pub fn is_null(&self) -> bool {
        let ptr = unsafe { T::__get(self.repr) };
        ptr.is_null()
    }

    /// Returns a reference to the object owned by this UniquePtr if any,
    /// otherwise None.
    pub fn as_ref(&self) -> Option<&T> {
        unsafe { T::__get(self.repr).as_ref() }
    }

    /// Returns a mutable reference to the object owned by this UniquePtr if
    /// any, otherwise None.
    pub fn as_mut(&mut self) -> Option<&mut T> {
        unsafe { (T::__get(self.repr) as *mut T).as_mut() }
    }

    /// Consumes the UniquePtr, releasing its ownership of the heap-allocated T.
    ///
    /// Matches the behavior of [std::unique_ptr\<T\>::release](https://en.cppreference.com/w/cpp/memory/unique_ptr/release).
    pub fn into_raw(self) -> *mut T {
        let ptr = unsafe { T::__release(self.repr) };
        mem::forget(self);
        ptr
    }

    /// Constructs a UniquePtr retaking ownership of a pointer previously
    /// obtained from `into_raw`.
    ///
    /// # Safety
    ///
    /// This function is unsafe because improper use may lead to memory
    /// problems. For example a double-free may occur if the function is called
    /// twice on the same raw pointer.
    pub unsafe fn from_raw(raw: *mut T) -> Self {
        UniquePtr {
            repr: T::__raw(raw),
            ty: PhantomData,
        }
    }
}

unsafe impl<T> Send for UniquePtr<T> where T: Send + UniquePtrTarget {}
unsafe impl<T> Sync for UniquePtr<T> where T: Sync + UniquePtrTarget {}

impl<T> Drop for UniquePtr<T>
where
    T: UniquePtrTarget,
{
    fn drop(&mut self) {
        unsafe { T::__drop(self.repr) }
    }
}

impl<T> Deref for UniquePtr<T>
where
    T: UniquePtrTarget,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        match self.as_ref() {
            Some(target) => target,
            None => panic!("called deref on a null UniquePtr<{}>", T::__NAME),
        }
    }
}

impl<T> DerefMut for UniquePtr<T>
where
    T: UniquePtrTarget,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self.as_mut() {
            Some(target) => target,
            None => panic!("called deref_mut on a null UniquePtr<{}>", T::__NAME),
        }
    }
}

impl<T> Debug for UniquePtr<T>
where
    T: Debug + UniquePtrTarget,
{
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self.as_ref() {
            None => formatter.write_str("nullptr"),
            Some(value) => Debug::fmt(value, formatter),
        }
    }
}

impl<T> Display for UniquePtr<T>
where
    T: Display + UniquePtrTarget,
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
pub unsafe trait UniquePtrTarget {
    #[doc(hidden)]
    const __NAME: &'static dyn Display;
    #[doc(hidden)]
    fn __null() -> *mut c_void;
    #[doc(hidden)]
    fn __new(value: Self) -> *mut c_void
    where
        Self: Sized,
    {
        // Opaque C types do not get this method because they can never exist by
        // value on the Rust side of the bridge.
        let _ = value;
        unreachable!()
    }
    #[doc(hidden)]
    unsafe fn __raw(raw: *mut Self) -> *mut c_void;
    #[doc(hidden)]
    unsafe fn __get(repr: *mut c_void) -> *const Self;
    #[doc(hidden)]
    unsafe fn __release(repr: *mut c_void) -> *mut Self;
    #[doc(hidden)]
    unsafe fn __drop(repr: *mut c_void);
}

extern "C" {
    #[link_name = "cxxbridge03$unique_ptr$std$string$null"]
    fn unique_ptr_std_string_null(this: *mut *mut c_void);
    #[link_name = "cxxbridge03$unique_ptr$std$string$raw"]
    fn unique_ptr_std_string_raw(this: *mut *mut c_void, raw: *mut CxxString);
    #[link_name = "cxxbridge03$unique_ptr$std$string$get"]
    fn unique_ptr_std_string_get(this: *const *mut c_void) -> *const CxxString;
    #[link_name = "cxxbridge03$unique_ptr$std$string$release"]
    fn unique_ptr_std_string_release(this: *mut *mut c_void) -> *mut CxxString;
    #[link_name = "cxxbridge03$unique_ptr$std$string$drop"]
    fn unique_ptr_std_string_drop(this: *mut *mut c_void);
}

unsafe impl UniquePtrTarget for CxxString {
    const __NAME: &'static dyn Display = &"CxxString";
    fn __null() -> *mut c_void {
        let mut repr = ptr::null_mut::<c_void>();
        unsafe { unique_ptr_std_string_null(&mut repr) }
        repr
    }
    unsafe fn __raw(raw: *mut Self) -> *mut c_void {
        let mut repr = ptr::null_mut::<c_void>();
        unique_ptr_std_string_raw(&mut repr, raw);
        repr
    }
    unsafe fn __get(repr: *mut c_void) -> *const Self {
        unique_ptr_std_string_get(&repr)
    }
    unsafe fn __release(mut repr: *mut c_void) -> *mut Self {
        unique_ptr_std_string_release(&mut repr)
    }
    unsafe fn __drop(mut repr: *mut c_void) {
        unique_ptr_std_string_drop(&mut repr);
    }
}

unsafe impl<T> UniquePtrTarget for CxxVector<T>
where
    T: VectorElement + 'static,
{
    const __NAME: &'static dyn Display = &cxx_vector::TypeName::<T>::new();
    fn __null() -> *mut c_void {
        T::__unique_ptr_null()
    }
    unsafe fn __raw(raw: *mut Self) -> *mut c_void {
        T::__unique_ptr_raw(raw)
    }
    unsafe fn __get(repr: *mut c_void) -> *const Self {
        T::__unique_ptr_get(repr)
    }
    unsafe fn __release(repr: *mut c_void) -> *mut Self {
        T::__unique_ptr_release(repr)
    }
    unsafe fn __drop(repr: *mut c_void) {
        T::__unique_ptr_drop(repr);
    }
}

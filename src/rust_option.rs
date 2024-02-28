#![allow(missing_docs)]
#![allow(clippy::let_unit_value)]

#[cfg(feature = "alloc")]
use crate::private::RustString;
#[cfg(feature = "alloc")]
use crate::private::RustVec;
#[cfg(feature = "alloc")]
use alloc::boxed::Box;
#[cfg(feature = "alloc")]
use alloc::string::String;
#[cfg(feature = "alloc")]
use alloc::vec::Vec;
use core::mem::ManuallyDrop;
use core::ops::Deref;
use core::ops::DerefMut;
use core::pin::Pin;

mod private {
    pub trait Sealed {}
}
pub trait OptionTarget: private::Sealed {}

impl<T: Sized> private::Sealed for &T {}
impl<T: Sized> OptionTarget for &T {}

impl<T: Sized> private::Sealed for &mut T {}
impl<T: Sized> OptionTarget for &mut T {}

impl<T: Sized> private::Sealed for Pin<&mut T> {}
impl<T: Sized> OptionTarget for Pin<&mut T> {}

impl<T: Sized> private::Sealed for *const T {}
impl<T: Sized> OptionTarget for *const T {}

impl<T: Sized> private::Sealed for *mut T {}
impl<T: Sized> OptionTarget for *mut T {}

#[cfg(feature = "alloc")]
impl<T: Sized> private::Sealed for Box<T> {}
#[cfg(feature = "alloc")]
impl<T: Sized> OptionTarget for Box<T> {}

#[repr(C)]
union OptionInner<T: OptionTarget> {
    value: ManuallyDrop<T>,
    empty: usize,
}

impl<T: OptionTarget> OptionInner<T> {
    fn none() -> Self {
        Self { empty: 0 }
    }

    #[cfg(feature = "alloc")]
    fn new(t: T) -> Self {
        Self {
            value: ManuallyDrop::new(t),
        }
    }

    fn has_value(&self) -> bool {
        let _: () = assert_option_safe::<&T>();
        unsafe { self.empty != 0 }
    }

    fn into_inner_unchecked(mut self) -> T {
        let value = unsafe { ManuallyDrop::take(&mut self.value) };
        unsafe { core::ptr::write(&mut self as _, OptionInner::none()) };
        value
    }
}

impl<T: OptionTarget> Drop for OptionInner<T> {
    fn drop(&mut self) {
        if self.has_value() {
            unsafe { ManuallyDrop::drop(&mut self.value) };
        }
    }
}

// ABI compatible with C++ rust::Option<T> (not necessarily core::option::Option<T>).
#[repr(C)]
pub struct RustOption<T: OptionTarget> {
    inner: OptionInner<T>,
}

pub const fn assert_option_safe<T>() {
    struct __SizeCheck<U>(core::marker::PhantomData<U>);
    impl<U> __SizeCheck<U> {
        const _IS_NICHE: () =
            assert!(core::mem::size_of::<Option<U>>() == core::mem::size_of::<U>());
        const _IS_USIZE_SIZE: () =
            assert!(core::mem::size_of::<Option<U>>() == core::mem::size_of::<usize>());
    }
    // Force the constants to resolve (at compile time)
    let _: () = __SizeCheck::<T>::_IS_NICHE;
    let _: () = __SizeCheck::<T>::_IS_USIZE_SIZE;
}

impl<T: OptionTarget> RustOption<T> {
    pub fn new() -> Self {
        let _: () = assert_option_safe::<&mut T>();
        RustOption {
            inner: OptionInner::none(),
        }
    }

    pub fn value(&self) -> Option<&T> {
        if self.has_value() {
            unsafe { Some(self.inner.value.deref()) }
        } else {
            None
        }
    }

    pub fn into_value(self) -> Option<T> {
        if self.has_value() {
            unsafe { Some(self.into_inner_unchecked()) }
        } else {
            None
        }
    }

    pub fn has_value(&self) -> bool {
        self.inner.has_value()
    }

    pub fn set(&mut self, value: T) {
        self.inner = OptionInner {
            value: ManuallyDrop::new(value),
        }
    }

    pub unsafe fn into_inner_unchecked(self) -> T {
        self.inner.into_inner_unchecked()
    }

    pub unsafe fn as_ref_inner_unchecked(&self) -> &T {
        unsafe { self.inner.value.deref() }
    }

    pub unsafe fn as_ref_mut_inner_unchecked(&mut self) -> &mut T {
        unsafe { self.inner.value.deref_mut() }
    }
}

impl<'a, T> RustOption<&'a T>
where
    &'a T: OptionTarget,
{
    pub fn from_option_ref(other: Option<&'a T>) -> Self {
        let _: () = assert_option_safe::<&T>();
        unsafe { core::mem::transmute::<Option<&'a T>, RustOption<&'a T>>(other) }
    }

    pub fn into_option_ref(self) -> Option<&'a T> {
        let _: () = assert_option_safe::<&T>();
        unsafe { core::mem::transmute::<RustOption<&'a T>, Option<&'a T>>(self) }
    }

    pub fn as_mut_option_ref(&mut self) -> &mut Option<&'a T> {
        let _: () = assert_option_safe::<&T>();
        unsafe { &mut *(self as *mut RustOption<&'a T> as *mut Option<&'a T>) }
    }
}

impl<'a, T> RustOption<&'a mut T>
where
    &'a mut T: OptionTarget,
{
    pub fn from_option_mut(other: Option<&'a mut T>) -> Self {
        let _: () = assert_option_safe::<&mut T>();
        unsafe { core::mem::transmute::<Option<&'a mut T>, RustOption<&'a mut T>>(other) }
    }

    pub fn into_option_mut(self) -> Option<&'a mut T> {
        let _: () = assert_option_safe::<&mut T>();
        unsafe { core::mem::transmute::<RustOption<&'a mut T>, Option<&'a mut T>>(self) }
    }

    pub fn as_mut_option_mut(&mut self) -> &mut Option<&'a mut T> {
        let _: () = assert_option_safe::<&mut T>();
        unsafe { &mut *(self as *mut RustOption<&'a mut T> as *mut Option<&'a mut T>) }
    }

    pub fn as_mut_option_mut_improper(&mut self) -> &mut Option<&'a mut T> {
        let _: () = assert_option_safe::<&mut T>();
        unsafe { &mut *(self as *mut RustOption<&'a mut T> as *mut Option<&'a mut T>) }
    }
}

impl<'a, T> RustOption<Pin<&'a mut T>>
where
    Pin<&'a mut T>: OptionTarget,
{
    pub fn from_option_mut_pinned(other: Option<Pin<&'a mut T>>) -> Self {
        let _: () = assert_option_safe::<Pin<&mut T>>();
        unsafe { core::mem::transmute::<Option<Pin<&'a mut T>>, RustOption<Pin<&'a mut T>>>(other) }
    }

    pub fn into_option_mut_pinned(self) -> Option<Pin<&'a mut T>> {
        let _: () = assert_option_safe::<Pin<&mut T>>();
        unsafe { core::mem::transmute::<RustOption<Pin<&'a mut T>>, Option<Pin<&'a mut T>>>(self) }
    }

    pub fn as_mut_option_mut_pinned(&mut self) -> &mut Option<Pin<&'a mut T>> {
        let _: () = assert_option_safe::<Pin<&mut T>>();
        unsafe { &mut *(self as *mut RustOption<Pin<&'a mut T>> as *mut Option<Pin<&'a mut T>>) }
    }

    pub fn into_option_mut_improper_pinned(self) -> Option<Pin<&'a mut T>> {
        let _: () = assert_option_safe::<Pin<&mut T>>();
        unsafe { core::mem::transmute::<RustOption<Pin<&'a mut T>>, Option<Pin<&'a mut T>>>(self) }
    }

    pub fn as_mut_option_mut_improper_pinned(&mut self) -> &mut Option<Pin<&'a mut T>> {
        let _: () = assert_option_safe::<Pin<&mut T>>();
        unsafe { &mut *(self as *mut RustOption<Pin<&'a mut T>> as *mut Option<Pin<&'a mut T>>) }
    }
}

impl<T> RustOption<*const T>
where
    *const T: OptionTarget,
{
    /// SAFETY: self must have been constructed as `Option<Box<T>>`
    #[cfg(feature = "alloc")]
    pub unsafe fn into_option_box(self) -> Option<Box<T>> {
        let _: () = assert_option_safe::<Box<T>>();
        unsafe { core::mem::transmute::<RustOption<*const T>, Option<Box<T>>>(self) }
    }

    /// SAFETY: self must have been constructed as `Option<Box<T>>`
    #[cfg(feature = "alloc")]
    pub unsafe fn as_mut_option_box(&mut self) -> &mut Option<Box<T>> {
        let _: () = assert_option_safe::<Box<T>>();
        unsafe { &mut *(self as *mut RustOption<*const T> as *mut Option<Box<T>>) }
    }

    pub fn into_option_ref<'a>(self) -> Option<&'a T> {
        let _: () = assert_option_safe::<&T>();
        unsafe { core::mem::transmute::<RustOption<*const T>, Option<&'a T>>(self) }
    }

    pub fn as_mut_option_ref<'a>(&mut self) -> &mut Option<&'a T> {
        let _: () = assert_option_safe::<&T>();
        unsafe { &mut *(self as *mut RustOption<*const T> as *mut Option<&'a T>) }
    }
}

impl<T> RustOption<*mut T>
where
    *mut T: OptionTarget,
{
    #[cfg(feature = "alloc")]
    pub fn from_option_box(other: Option<Box<T>>) -> Self {
        let _: () = assert_option_safe::<Box<T>>();
        unsafe { core::mem::transmute::<Option<Box<T>>, RustOption<*mut T>>(other) }
    }

    /// SAFETY: self must have been constructed as `Option<Box<T>>`
    #[cfg(feature = "alloc")]
    pub unsafe fn into_option_box(self) -> Option<Box<T>> {
        let _: () = assert_option_safe::<Box<T>>();
        unsafe { core::mem::transmute::<RustOption<*mut T>, Option<Box<T>>>(self) }
    }

    /// SAFETY: self must have been constructed as `Option<Box<T>>`
    #[cfg(feature = "alloc")]
    pub unsafe fn as_mut_option_box(&mut self) -> &mut Option<Box<T>> {
        let _: () = assert_option_safe::<Box<T>>();
        unsafe { &mut *(self as *mut RustOption<*mut T> as *mut Option<Box<T>>) }
    }

    /// SAFETY: Pointer must not be aliased and must have been constructed as an `Option<&mut T>`
    pub unsafe fn into_option_mut<'a>(self) -> Option<&'a mut T> {
        let _: () = assert_option_safe::<&mut T>();
        unsafe { core::mem::transmute::<RustOption<*mut T>, Option<&'a mut T>>(self) }
    }

    /// SAFETY: Pointer must not be aliased and must have been constructed as an `Option<&mut T>`
    pub unsafe fn as_mut_option_mut<'a>(&mut self) -> &mut Option<&'a mut T> {
        let _: () = assert_option_safe::<&mut T>();
        unsafe { &mut *(self as *mut RustOption<*mut T> as *mut Option<&'a mut T>) }
    }
}

impl RustOption<*const core::ffi::c_void> {
    pub fn from_option_ref_improper<'a, T>(other: Option<&'a T>) -> Self {
        let _: () = assert_option_safe::<&T>();
        unsafe {
            core::mem::transmute::<Option<&'a T>, RustOption<*const core::ffi::c_void>>(other)
        }
    }

    pub fn from_option_ref_improper_pinned<'a, T>(other: Option<Pin<&'a T>>) -> Self {
        let _: () = assert_option_safe::<Pin<&T>>();
        unsafe {
            core::mem::transmute::<Option<Pin<&'a T>>, RustOption<*const core::ffi::c_void>>(other)
        }
    }

    /// SAFETY: self must have been constructed as `Option<Box<T>>`
    #[cfg(feature = "alloc")]
    pub unsafe fn into_option_box_improper<T>(self) -> Option<Box<T>> {
        let _: () = assert_option_safe::<Box<T>>();
        unsafe {
            core::mem::transmute::<RustOption<*const core::ffi::c_void>, Option<Box<T>>>(self)
        }
    }

    /// SAFETY: self must have been constructed as `Option<Box<T>>`
    #[cfg(feature = "alloc")]
    pub unsafe fn as_mut_option_box_improper<T>(&mut self) -> &mut Option<Box<T>> {
        let _: () = assert_option_safe::<Box<T>>();
        unsafe { &mut *(self as *mut RustOption<*const core::ffi::c_void> as *mut Option<Box<T>>) }
    }

    pub fn into_option_ref_improper<'a, T>(self) -> Option<&'a T> {
        let _: () = assert_option_safe::<&T>();
        unsafe { core::mem::transmute::<RustOption<*const core::ffi::c_void>, Option<&'a T>>(self) }
    }

    pub fn as_mut_option_ref_improper<'a, T>(&mut self) -> &mut Option<&'a T> {
        let _: () = assert_option_safe::<&T>();
        unsafe { &mut *(self as *mut RustOption<*const core::ffi::c_void> as *mut Option<&'a T>) }
    }
}

impl RustOption<*mut core::ffi::c_void> {
    #[cfg(feature = "alloc")]
    pub fn from_option_box_improper<T>(other: Option<Box<T>>) -> Self {
        let _: () = assert_option_safe::<Box<T>>();
        unsafe { core::mem::transmute::<Option<Box<T>>, RustOption<*mut core::ffi::c_void>>(other) }
    }

    pub fn from_option_mut_improper<'a, T>(other: Option<&'a mut T>) -> Self {
        let _: () = assert_option_safe::<&mut T>();
        unsafe {
            core::mem::transmute::<Option<&'a mut T>, RustOption<*mut core::ffi::c_void>>(other)
        }
    }

    pub fn from_option_mut_improper_pinned<'a, T>(other: Option<Pin<&'a mut T>>) -> Self {
        let _: () = assert_option_safe::<Pin<&mut T>>();
        unsafe {
            core::mem::transmute::<Option<Pin<&'a mut T>>, RustOption<*mut core::ffi::c_void>>(
                other,
            )
        }
    }

    /// SAFETY: self must have been constructed as `Option<Box<T>>`
    #[cfg(feature = "alloc")]
    pub unsafe fn into_option_box_improper<T>(self) -> Option<Box<T>> {
        let _: () = assert_option_safe::<Box<T>>();
        unsafe { core::mem::transmute::<RustOption<*mut core::ffi::c_void>, Option<Box<T>>>(self) }
    }

    /// SAFETY: self must have been constructed as `Option<Box<T>>`
    #[cfg(feature = "alloc")]
    pub unsafe fn as_mut_option_box_improper<T>(&mut self) -> &mut Option<Box<T>> {
        let _: () = assert_option_safe::<Box<T>>();
        unsafe { &mut *(self as *mut RustOption<*mut core::ffi::c_void> as *mut Option<Box<T>>) }
    }

    /// SAFETY: Pointer must not be aliased and must have been constructed as an `Option<&mut T>`
    pub unsafe fn into_option_mut_improper<'a, T>(self) -> Option<&'a mut T> {
        let _: () = assert_option_safe::<&mut T>();
        unsafe {
            core::mem::transmute::<RustOption<*mut core::ffi::c_void>, Option<&'a mut T>>(self)
        }
    }

    /// SAFETY: Pointer must not be aliased and must have been constructed as an `Option<&mut T>`
    pub unsafe fn as_mut_option_mut_improper<'a, T>(&mut self) -> &mut Option<&'a mut T> {
        let _: () = assert_option_safe::<&mut T>();
        unsafe { &mut *(self as *mut RustOption<*mut core::ffi::c_void> as *mut Option<&'a mut T>) }
    }
}

#[cfg(feature = "alloc")]
impl<'a, T> RustOption<&'a RustVec<T>> {
    pub fn from_option_vec_ref(other: Option<&'a Vec<T>>) -> Self {
        let _: () = assert_option_safe::<&'a Vec<T>>();
        match other {
            None => Self::new(),
            Some(r) => Self {
                inner: OptionInner::new(RustVec::from_ref(r)),
            },
        }
    }

    pub fn into_option_vec_ref(self) -> Option<&'a Vec<T>> {
        let _: () = assert_option_safe::<&Vec<T>>();
        match self.into_value() {
            None => None,
            Some(r) => Some(r.as_vec()),
        }
    }

    pub fn as_option_vec_ref(&mut self) -> &mut Option<&'a Vec<T>> {
        let _: () = assert_option_safe::<&Vec<T>>();
        unsafe { &mut *(self as *mut RustOption<&'a RustVec<T>> as *mut Option<&'a Vec<T>>) }
    }
}

#[cfg(feature = "alloc")]
impl<'a, T> RustOption<&'a mut RustVec<T>> {
    pub fn from_option_vec_mut(other: Option<&'a mut Vec<T>>) -> Self {
        let _: () = assert_option_safe::<&mut Vec<T>>();
        match other {
            None => Self::new(),
            Some(r) => Self {
                inner: OptionInner::new(RustVec::from_mut(r)),
            },
        }
    }

    pub fn into_option_vec_mut(self) -> Option<&'a mut Vec<T>> {
        let _: () = assert_option_safe::<&mut Vec<T>>();
        match self.into_value() {
            None => None,
            Some(r) => Some(r.as_mut_vec()),
        }
    }

    pub fn as_option_vec_mut(&mut self) -> &mut Option<&'a mut Vec<T>> {
        let _: () = assert_option_safe::<&mut Vec<T>>();
        unsafe {
            &mut *(self as *mut RustOption<&'a mut RustVec<T>> as *mut Option<&'a mut Vec<T>>)
        }
    }
}

#[cfg(feature = "alloc")]
impl<'a> RustOption<&'a RustVec<RustString>> {
    pub fn from_option_vec_string_ref(other: Option<&'a Vec<String>>) -> Self {
        let _: () = assert_option_safe::<&Vec<String>>();
        match other {
            None => Self::new(),
            Some(r) => Self {
                inner: OptionInner::new(RustVec::from_ref_vec_string(r)),
            },
        }
    }

    pub fn into_option_vec_string_ref(self) -> Option<&'a Vec<String>> {
        let _: () = assert_option_safe::<&Vec<String>>();
        match self.into_value() {
            None => None,
            Some(r) => Some(r.as_vec_string()),
        }
    }

    pub fn as_option_vec_string_ref(&mut self) -> &mut Option<&'a Vec<String>> {
        let _: () = assert_option_safe::<&Vec<String>>();
        unsafe {
            &mut *(self as *mut RustOption<&'a RustVec<RustString>> as *mut Option<&'a Vec<String>>)
        }
    }
}

#[cfg(feature = "alloc")]
impl<'a> RustOption<&'a mut RustVec<RustString>> {
    pub fn from_option_vec_string_mut(other: Option<&'a mut Vec<String>>) -> Self {
        let _: () = assert_option_safe::<&mut Vec<String>>();
        match other {
            None => Self::new(),
            Some(r) => Self {
                inner: OptionInner::new(RustVec::from_mut_vec_string(r)),
            },
        }
    }

    pub fn into_option_vec_string_mut(self) -> Option<&'a mut Vec<String>> {
        let _: () = assert_option_safe::<&mut Vec<String>>();
        match self.into_value() {
            None => None,
            Some(r) => Some(r.as_mut_vec_string()),
        }
    }

    pub fn as_option_vec_string_mut(&mut self) -> &mut Option<&'a mut Vec<String>> {
        let _: () = assert_option_safe::<&mut Vec<String>>();
        unsafe {
            &mut *(self as *mut RustOption<&'a mut RustVec<RustString>>
                as *mut Option<&'a mut Vec<String>>)
        }
    }
}

#[cfg(feature = "alloc")]
impl<'a> RustOption<&'a RustString> {
    pub fn from_option_string_ref(other: Option<&'a String>) -> Self {
        let _: () = assert_option_safe::<&String>();
        match other {
            None => Self::new(),
            Some(r) => Self {
                inner: OptionInner::new(RustString::from_ref(r)),
            },
        }
    }

    pub fn into_option_string_ref(self) -> Option<&'a String> {
        let _: () = assert_option_safe::<&String>();
        match self.into_value() {
            None => None,
            Some(r) => Some(r.as_string()),
        }
    }

    pub fn as_option_string_ref(&mut self) -> &mut Option<&'a String> {
        let _: () = assert_option_safe::<&String>();
        unsafe { &mut *(self as *mut RustOption<&'a RustString> as *mut Option<&'a String>) }
    }
}

#[cfg(feature = "alloc")]
impl<'a> RustOption<&'a mut RustString> {
    pub fn from_option_string_mut(other: Option<&'a mut String>) -> Self {
        let _: () = assert_option_safe::<&mut String>();
        match other {
            None => Self::new(),
            Some(r) => Self {
                inner: OptionInner::new(RustString::from_mut(r)),
            },
        }
    }

    pub fn into_option_string_mut(self) -> Option<&'a mut String> {
        let _: () = assert_option_safe::<&mut String>();
        match self.into_value() {
            None => None,
            Some(r) => Some(r.as_mut_string()),
        }
    }

    pub fn as_option_string_mut(&mut self) -> &mut Option<&'a mut String> {
        let _: () = assert_option_safe::<&mut String>();
        unsafe {
            &mut *(self as *mut RustOption<&'a mut RustString> as *mut Option<&'a mut String>)
        }
    }
}

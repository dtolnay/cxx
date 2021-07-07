#![allow(missing_docs)]
#![allow(clippy::let_unit_value)]

#[cfg(feature = "alloc")]
use alloc::boxed::Box;
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
    fn has_value(&self) -> bool {
        let _: () = assert_option_safe::<&T>();
        unsafe { self.empty != 0 }
    }

    fn into_inner_unchecked(mut self) -> T {
        let value = unsafe { ManuallyDrop::take(&mut self.value) };
        unsafe { core::ptr::write(&mut self as _, OptionInner { empty: 0 }) };
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
            inner: OptionInner { empty: 0 },
        }
    }

    pub fn value(&self) -> Option<&T> {
        if self.has_value() {
            unsafe { Some(self.inner.value.deref()) }
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

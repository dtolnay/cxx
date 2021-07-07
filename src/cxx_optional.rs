//! Less used details of `CxxVector` are exposed in this module. `CxxVector`
//! itself is exposed at the crate root.

use crate::string::CxxString;
use core::ffi::c_void;
use core::fmt::{self, Debug};
use core::iter::FusedIterator;
use core::marker::{PhantomData, PhantomPinned};
use core::mem::{self, MaybeUninit};
use core::pin::Pin;

/// Binding to C++ `std::optional<T>`.
///
/// # Invariants
///
/// As an invariant of this API and the static analysis of the cxx::bridge
/// macro, in Rust code we can never obtain a `CxxOptional` by value. Instead in
/// Rust code we will only ever look at a vector behind a reference or smart
/// pointer, as in `&CxxOptional<T>` or `UniquePtr<CxxOptional<T>>`.
#[repr(C, packed)]
pub struct CxxOptional<T> {
    // A thing, because repr(C) structs are not allowed to consist exclusively
    // of PhantomData fields.
    _void: [c_void; 0],
    // The conceptual optional elements to ensure that autotraits are propagated
    // correctly, e.g. CxxOptional is UnwindSafe iff T is.
    _elements: PhantomData<T>,
    // Prevent unpin operation from Pin<&mut CxxOptional<T>> to &mut CxxOptional<T>.
    _pinned: PhantomData<PhantomPinned>,
}

impl<T> CxxOptional<T>
where
    T: OptionalElement,
{
    /// Returns a reference to the value in the optional.
    pub fn value(&self) -> Option<&T> {
        if T::__has_value(self) {
            unsafe { Some(T::__value(self)) }
        } else {
            None
        }
    }

    /// Returns a reference to the value in the optional.
    pub fn value_mut(self: Pin<&mut Self>) -> Option<Pin<&mut T>> {
        let o_ref = unsafe { self.get_unchecked_mut() };
        let this = o_ref as *mut Self;
        if T::__has_value(this) {
            unsafe { Some(Pin::new_unchecked(T::__value_mut(o_ref))) }
        } else {
            None
        }
    }

    /// Returns a reference to the value in the optional without checking.
    ///
    /// SAFETY: You must ensure the value is `Some`
    pub unsafe fn value_unchecked(&self) -> &T {
        T::__value(self)
    }

    /// Returns a mutable reference to the value in the optional without checking.
    ///
    /// SAFETY: You must ensure the value is `Some`
    pub unsafe fn value_mut_unchecked(&mut self) -> &mut T {
        T::__value_mut(self)
    }

    /// Returns a const iterator yielding the element inside if it exists
    pub fn iter<'a>(&'a self) -> Iter<'a, T> {
        Iter {
            o: self,
            consumed: false,
        }
    }

    /// Returns a mutable iterator yielding the element inside if it exists
    pub fn iter_mut<'a>(self: Pin<&'a mut Self>) -> IterMut<'a, T> {
        IterMut {
            o: self,
            consumed: false,
        }
    }
}

/// Iterator over elements of a `CxxOptional` by shared reference.
///
/// The iterator element type is `&'a T`.
pub struct Iter<'a, T> {
    o: &'a CxxOptional<T>,
    consumed: bool,
}

impl<'a, T> IntoIterator for &'a CxxOptional<T>
where
    T: OptionalElement,
{
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T> Iterator for Iter<'a, T>
where
    T: OptionalElement,
{
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.consumed {
            None
        } else {
            self.consumed = true;
            self.o.value()
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        if self.o.value().is_some() && !self.consumed {
            (1, Some(1))
        } else {
            (0, Some(0))
        }
    }
}

impl<'a, T> ExactSizeIterator for Iter<'a, T>
where
    T: OptionalElement,
{
    fn len(&self) -> usize {
        if self.o.value().is_some() && !self.consumed {
            1
        } else {
            0
        }
    }
}

impl<'a, T> FusedIterator for Iter<'a, T> where T: OptionalElement {}

/// Iterator over elements of a `CxxOptional` by pinned mutable reference.
///
/// The iterator element type is `Pin<&'a mut T>`.
pub struct IterMut<'a, T> {
    o: Pin<&'a mut CxxOptional<T>>,
    consumed: bool,
}

impl<'a, T> IntoIterator for Pin<&'a mut CxxOptional<T>>
where
    T: OptionalElement,
{
    type Item = Pin<&'a mut T>;
    type IntoIter = IterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<'a, T> Iterator for IterMut<'a, T>
where
    T: OptionalElement,
{
    type Item = Pin<&'a mut T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.consumed {
            None
        } else {
            self.consumed = true;
            let o_ref = self.o.as_mut();
            let value = o_ref.value_mut()?;
            // Extend lifetime to allow simultaneous holding of nonoverlapping
            // elements, analogous to slice::split_first_mut.
            unsafe {
                let ptr = Pin::into_inner_unchecked(value) as *mut T;
                Some(Pin::new_unchecked(&mut *ptr))
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        if self.o.value().is_some() && !self.consumed {
            (1, Some(1))
        } else {
            (0, Some(0))
        }
    }
}

impl<'a, T> ExactSizeIterator for IterMut<'a, T>
where
    T: OptionalElement,
{
    fn len(&self) -> usize {
        if self.o.value().is_some() && !self.consumed {
            1
        } else {
            0
        }
    }
}

impl<'a, T> FusedIterator for IterMut<'a, T> where T: OptionalElement {}

impl<T> Debug for CxxOptional<T>
where
    T: OptionalElement + Debug,
{
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.debug_list().entries(self).finish()
    }
}

/// Trait bound for types which may be used as the `T` inside of a
/// `CxxOptional<T>` in generic code.
///
/// This trait has no publicly callable or implementable methods. Implementing
/// it outside of the CXX codebase is not supported.
///
/// # Example
///
/// A bound `T: OptionalElement` may be necessary when manipulating [`CxxOptional`]
/// in generic code.
///
/// ```
/// use cxx::vector::{CxxOptional, OptionalElement};
/// use std::fmt::Display;
///
/// pub fn take_generic_vector<T>(vector: &CxxOptional<T>)
/// where
///     T: OptionalElement + Display,
/// {
///     println!("the vector elements are:");
///     for element in vector {
///         println!("  • {}", element);
///     }
/// }
/// ```
///
/// Writing the same generic function without a `OptionalElement` trait bound
/// would not compile.
pub unsafe trait OptionalElement: Sized {
    #[doc(hidden)]
    fn __typename(f: &mut fmt::Formatter) -> fmt::Result;
    #[doc(hidden)]
    fn __has_value(o: *const CxxOptional<Self>) -> bool;
    #[doc(hidden)]
    unsafe fn __value<'a>(v: &'a CxxOptional<Self>) -> &'a Self;
    #[doc(hidden)]
    unsafe fn __value_mut<'a>(v: &'a mut CxxOptional<Self>) -> &'a mut Self;
    #[doc(hidden)]
    fn __unique_ptr_null() -> MaybeUninit<*mut c_void>;
    #[doc(hidden)]
    unsafe fn __unique_ptr_raw(raw: *mut CxxOptional<Self>) -> MaybeUninit<*mut c_void>;
    #[doc(hidden)]
    unsafe fn __unique_ptr_get(repr: MaybeUninit<*mut c_void>) -> *const CxxOptional<Self>;
    #[doc(hidden)]
    unsafe fn __unique_ptr_release(repr: MaybeUninit<*mut c_void>) -> *mut CxxOptional<Self>;
    #[doc(hidden)]
    unsafe fn __unique_ptr_drop(repr: MaybeUninit<*mut c_void>);
}

macro_rules! impl_optional_element {
    ($kind:ident, $segment:expr, $name:expr, $ty:ty) => {
        const_assert_eq!(0, mem::size_of::<CxxOptional<$ty>>());
        const_assert_eq!(1, mem::align_of::<CxxOptional<$ty>>());

        unsafe impl OptionalElement for $ty {
            #[doc(hidden)]
            fn __typename(f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str($name)
            }
            #[doc(hidden)]
            fn __has_value(o: *const CxxOptional<$ty>) -> bool {
                extern "C" {
                    attr! {
                        #[link_name = concat!("cxxbridge1$std$optional$", $segment, "$has_value")]
                        fn __has_value(_: *const CxxOptional<$ty>) -> bool;
                    }
                }
                unsafe { __has_value(o) }
            }
            #[doc(hidden)]
            unsafe fn __value<'a>(o: &'a CxxOptional<$ty>) -> &'a $ty {
                extern "C" {
                    attr! {
                        #[link_name = concat!("cxxbridge1$std$optional$", $segment, "$value")]
                        fn __value(_: *const CxxOptional<$ty>) -> *const $ty;
                    }
                }
                __value(o as *const CxxOptional<$ty>).as_ref().unwrap()
            }
            #[doc(hidden)]
            unsafe fn __value_mut<'a>(o: &'a mut CxxOptional<$ty>) -> &'a mut $ty {
                extern "C" {
                    attr! {
                        #[link_name = concat!("cxxbridge1$std$optional$", $segment, "$value_mut")]
                        fn __value_mut(_: *mut CxxOptional<$ty>) -> *mut $ty;
                    }
                }
                __value_mut(o as *mut CxxOptional<$ty>).as_mut().unwrap()
            }
            #[doc(hidden)]
            fn __unique_ptr_null() -> MaybeUninit<*mut c_void> {
                extern "C" {
                    attr! {
                        #[link_name = concat!("cxxbridge1$unique_ptr$std$optional$", $segment, "$null")]
                        fn __unique_ptr_null(this: *mut MaybeUninit<*mut c_void>);
                    }
                }
                let mut repr = MaybeUninit::uninit();
                unsafe { __unique_ptr_null(&mut repr) }
                repr
            }
            #[doc(hidden)]
            unsafe fn __unique_ptr_raw(raw: *mut CxxOptional<Self>) -> MaybeUninit<*mut c_void> {
                extern "C" {
                    attr! {
                        #[link_name = concat!("cxxbridge1$unique_ptr$std$optional$", $segment, "$raw")]
                        fn __unique_ptr_raw(this: *mut MaybeUninit<*mut c_void>, raw: *mut CxxOptional<$ty>);
                    }
                }
                let mut repr = MaybeUninit::uninit();
                __unique_ptr_raw(&mut repr, raw);
                repr
            }
            #[doc(hidden)]
            unsafe fn __unique_ptr_get(repr: MaybeUninit<*mut c_void>) -> *const CxxOptional<Self> {
                extern "C" {
                    attr! {
                        #[link_name = concat!("cxxbridge1$unique_ptr$std$optional$", $segment, "$get")]
                        fn __unique_ptr_get(this: *const MaybeUninit<*mut c_void>) -> *const CxxOptional<$ty>;
                    }
                }
                __unique_ptr_get(&repr)
            }
            #[doc(hidden)]
            unsafe fn __unique_ptr_release(mut repr: MaybeUninit<*mut c_void>) -> *mut CxxOptional<Self> {
                extern "C" {
                    attr! {
                        #[link_name = concat!("cxxbridge1$unique_ptr$std$optional$", $segment, "$release")]
                        fn __unique_ptr_release(this: *mut MaybeUninit<*mut c_void>) -> *mut CxxOptional<$ty>;
                    }
                }
                __unique_ptr_release(&mut repr)
            }
            #[doc(hidden)]
            unsafe fn __unique_ptr_drop(mut repr: MaybeUninit<*mut c_void>) {
                extern "C" {
                    attr! {
                        #[link_name = concat!("cxxbridge1$unique_ptr$std$optional$", $segment, "$drop")]
                        fn __unique_ptr_drop(this: *mut MaybeUninit<*mut c_void>);
                    }
                }
                __unique_ptr_drop(&mut repr);
            }
        }
    };
}

macro_rules! impl_optional_element_for_primitive {
    ($ty:ident) => {
        impl_optional_element!(trivial, stringify!($ty), stringify!($ty), $ty);
    };
}

impl_optional_element_for_primitive!(u8);
impl_optional_element_for_primitive!(u16);
impl_optional_element_for_primitive!(u32);
impl_optional_element_for_primitive!(u64);
impl_optional_element_for_primitive!(usize);
impl_optional_element_for_primitive!(i8);
impl_optional_element_for_primitive!(i16);
impl_optional_element_for_primitive!(i32);
impl_optional_element_for_primitive!(i64);
impl_optional_element_for_primitive!(isize);
impl_optional_element_for_primitive!(f32);
impl_optional_element_for_primitive!(f64);

impl_optional_element!(opaque, "string", "CxxString", CxxString);

//! Less used details of `CxxVector` are exposed in this module. `CxxVector`
//! itself is exposed at the crate root.

use crate::extern_type::ExternType;
use crate::kind::Trivial;
use crate::string::CxxString;
use core::ffi::c_void;
use core::fmt::{self, Debug};
use core::iter::FusedIterator;
use core::marker::{PhantomData, PhantomPinned};
use core::mem;
use core::pin::Pin;
use core::ptr;
use core::slice;

/// Binding to C++ `std::vector<T, std::allocator<T>>`.
///
/// # Invariants
///
/// As an invariant of this API and the static analysis of the cxx::bridge
/// macro, in Rust code we can never obtain a `CxxVector` by value. Instead in
/// Rust code we will only ever look at a vector behind a reference or smart
/// pointer, as in `&CxxVector<T>` or `UniquePtr<CxxVector<T>>`.
#[repr(C, packed)]
pub struct CxxVector<T> {
    _private: [T; 0],
    _pinned: PhantomData<PhantomPinned>,
}

impl<T> CxxVector<T>
where
    T: VectorElement,
{
    /// Returns the number of elements in the vector.
    ///
    /// Matches the behavior of C++ [std::vector\<T\>::size][size].
    ///
    /// [size]: https://en.cppreference.com/w/cpp/container/vector/size
    pub fn len(&self) -> usize {
        T::__vector_size(self)
    }

    /// Returns true if the vector contains no elements.
    ///
    /// Matches the behavior of C++ [std::vector\<T\>::empty][empty].
    ///
    /// [empty]: https://en.cppreference.com/w/cpp/container/vector/empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns a reference to an element at the given position, or `None` if
    /// out of bounds.
    pub fn get(&self, pos: usize) -> Option<&T> {
        if pos < self.len() {
            Some(unsafe { self.get_unchecked(pos) })
        } else {
            None
        }
    }

    /// Returns a pinned mutable reference to an element at the given position,
    /// or `None` if out of bounds.
    pub fn index_mut(self: Pin<&mut Self>, pos: usize) -> Option<Pin<&mut T>> {
        if pos < self.len() {
            Some(unsafe { self.index_unchecked_mut(pos) })
        } else {
            None
        }
    }

    /// Returns a reference to an element without doing bounds checking.
    ///
    /// This is generally not recommended, use with caution! Calling this method
    /// with an out-of-bounds index is undefined behavior even if the resulting
    /// reference is not used.
    ///
    /// Matches the behavior of C++
    /// [std::vector\<T\>::operator\[\] const][operator_at].
    ///
    /// [operator_at]: https://en.cppreference.com/w/cpp/container/vector/operator_at
    pub unsafe fn get_unchecked(&self, pos: usize) -> &T {
        let this = self as *const CxxVector<T> as *mut CxxVector<T>;
        let ptr = T::__get_unchecked(this, pos) as *const T;
        &*ptr
    }

    /// Returns a pinned mutable reference to an element without doing bounds
    /// checking.
    ///
    /// This is generally not recommended, use with caution! Calling this method
    /// with an out-of-bounds index is undefined behavior even if the resulting
    /// reference is not used.
    ///
    /// Matches the behavior of C++
    /// [std::vector\<T\>::operator\[\]][operator_at].
    ///
    /// [operator_at]: https://en.cppreference.com/w/cpp/container/vector/operator_at
    pub unsafe fn index_unchecked_mut(self: Pin<&mut Self>, pos: usize) -> Pin<&mut T> {
        let ptr = T::__get_unchecked(self.get_unchecked_mut(), pos);
        Pin::new_unchecked(&mut *ptr)
    }

    /// Returns a slice to the underlying contiguous array of elements.
    pub fn as_slice(&self) -> &[T]
    where
        T: ExternType<Kind = Trivial>,
    {
        let len = self.len();
        if len == 0 {
            // The slice::from_raw_parts in the other branch requires a nonnull
            // and properly aligned data ptr. C++ standard does not guarantee
            // that data() on a vector with size 0 would return a nonnull
            // pointer or sufficiently aligned pointer, so using it would be
            // undefined behavior. Create our own empty slice in Rust instead
            // which upholds the invariants.
            &[]
        } else {
            let this = self as *const CxxVector<T> as *mut CxxVector<T>;
            let ptr = unsafe { T::__get_unchecked(this, 0) };
            unsafe { slice::from_raw_parts(ptr, len) }
        }
    }

    /// Returns a slice to the underlying contiguous array of elements by
    /// mutable reference.
    pub fn as_mut_slice(self: Pin<&mut Self>) -> &mut [T]
    where
        T: ExternType<Kind = Trivial>,
    {
        let len = self.len();
        if len == 0 {
            &mut []
        } else {
            let ptr = unsafe { T::__get_unchecked(self.get_unchecked_mut(), 0) };
            unsafe { slice::from_raw_parts_mut(ptr, len) }
        }
    }

    /// Returns an iterator over elements of type `&T`.
    pub fn iter(&self) -> Iter<T> {
        Iter { v: self, index: 0 }
    }

    /// Returns an iterator over elements of type `Pin<&mut T>`.
    pub fn iter_mut(self: Pin<&mut Self>) -> IterMut<T> {
        IterMut { v: self, index: 0 }
    }
}

/// Iterator over elements of a `CxxVector` by shared reference.
///
/// The iterator element type is `&'a T`.
pub struct Iter<'a, T> {
    v: &'a CxxVector<T>,
    index: usize,
}

impl<'a, T> IntoIterator for &'a CxxVector<T>
where
    T: VectorElement,
{
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T> Iterator for Iter<'a, T>
where
    T: VectorElement,
{
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.v.get(self.index)?;
        self.index += 1;
        Some(next)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }
}

impl<'a, T> ExactSizeIterator for Iter<'a, T>
where
    T: VectorElement,
{
    fn len(&self) -> usize {
        self.v.len() - self.index
    }
}

impl<'a, T> FusedIterator for Iter<'a, T> where T: VectorElement {}

/// Iterator over elements of a `CxxVector` by pinned mutable reference.
///
/// The iterator element type is `Pin<&'a mut T>`.
pub struct IterMut<'a, T> {
    v: Pin<&'a mut CxxVector<T>>,
    index: usize,
}

impl<'a, T> IntoIterator for Pin<&'a mut CxxVector<T>>
where
    T: VectorElement,
{
    type Item = Pin<&'a mut T>;
    type IntoIter = IterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<'a, T> Iterator for IterMut<'a, T>
where
    T: VectorElement,
{
    type Item = Pin<&'a mut T>;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.v.as_mut().index_mut(self.index)?;
        self.index += 1;
        // Extend lifetime to allow simultaneous holding of nonoverlapping
        // elements, analogous to slice::split_first_mut.
        unsafe {
            let ptr = Pin::into_inner_unchecked(next) as *mut T;
            Some(Pin::new_unchecked(&mut *ptr))
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }
}

impl<'a, T> ExactSizeIterator for IterMut<'a, T>
where
    T: VectorElement,
{
    fn len(&self) -> usize {
        self.v.len() - self.index
    }
}

impl<'a, T> FusedIterator for IterMut<'a, T> where T: VectorElement {}

impl<T> Debug for CxxVector<T>
where
    T: VectorElement + Debug,
{
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.debug_list().entries(self).finish()
    }
}

/// Trait bound for types which may be used as the `T` inside of a
/// `CxxVector<T>` in generic code.
///
/// This trait has no publicly callable or implementable methods. Implementing
/// it outside of the CXX codebase is not supported.
///
/// # Example
///
/// A bound `T: VectorElement` may be necessary when manipulating [`CxxVector`]
/// in generic code.
///
/// ```
/// use cxx::vector::{CxxVector, VectorElement};
/// use std::fmt::Display;
///
/// pub fn take_generic_vector<T>(vector: &CxxVector<T>)
/// where
///     T: VectorElement + Display,
/// {
///     println!("the vector elements are:");
///     for element in vector {
///         println!("  • {}", element);
///     }
/// }
/// ```
///
/// Writing the same generic function without a `VectorElement` trait bound
/// would not compile.
pub unsafe trait VectorElement: Sized {
    #[doc(hidden)]
    fn __typename(f: &mut fmt::Formatter) -> fmt::Result;
    #[doc(hidden)]
    fn __vector_size(v: &CxxVector<Self>) -> usize;
    #[doc(hidden)]
    unsafe fn __get_unchecked(v: *mut CxxVector<Self>, pos: usize) -> *mut Self;
    #[doc(hidden)]
    fn __unique_ptr_null() -> *mut c_void;
    #[doc(hidden)]
    unsafe fn __unique_ptr_raw(raw: *mut CxxVector<Self>) -> *mut c_void;
    #[doc(hidden)]
    unsafe fn __unique_ptr_get(repr: *mut c_void) -> *const CxxVector<Self>;
    #[doc(hidden)]
    unsafe fn __unique_ptr_release(repr: *mut c_void) -> *mut CxxVector<Self>;
    #[doc(hidden)]
    unsafe fn __unique_ptr_drop(repr: *mut c_void);
}

macro_rules! impl_vector_element {
    ($segment:expr, $name:expr, $ty:ty) => {
        const_assert_eq!(1, mem::align_of::<CxxVector<$ty>>());

        unsafe impl VectorElement for $ty {
            #[doc(hidden)]
            fn __typename(f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str($name)
            }
            #[doc(hidden)]
            fn __vector_size(v: &CxxVector<$ty>) -> usize {
                extern "C" {
                    attr! {
                        #[link_name = concat!("cxxbridge1$std$vector$", $segment, "$size")]
                        fn __vector_size(_: &CxxVector<$ty>) -> usize;
                    }
                }
                unsafe { __vector_size(v) }
            }
            #[doc(hidden)]
            unsafe fn __get_unchecked(v: *mut CxxVector<$ty>, pos: usize) -> *mut $ty {
                extern "C" {
                    attr! {
                        #[link_name = concat!("cxxbridge1$std$vector$", $segment, "$get_unchecked")]
                        fn __get_unchecked(_: *mut CxxVector<$ty>, _: usize) -> *mut $ty;
                    }
                }
                __get_unchecked(v, pos)
            }
            #[doc(hidden)]
            fn __unique_ptr_null() -> *mut c_void {
                extern "C" {
                    attr! {
                        #[link_name = concat!("cxxbridge1$unique_ptr$std$vector$", $segment, "$null")]
                        fn __unique_ptr_null(this: *mut *mut c_void);
                    }
                }
                let mut repr = ptr::null_mut::<c_void>();
                unsafe { __unique_ptr_null(&mut repr) }
                repr
            }
            #[doc(hidden)]
            unsafe fn __unique_ptr_raw(raw: *mut CxxVector<Self>) -> *mut c_void {
                extern "C" {
                    attr! {
                        #[link_name = concat!("cxxbridge1$unique_ptr$std$vector$", $segment, "$raw")]
                        fn __unique_ptr_raw(this: *mut *mut c_void, raw: *mut CxxVector<$ty>);
                    }
                }
                let mut repr = ptr::null_mut::<c_void>();
                __unique_ptr_raw(&mut repr, raw);
                repr
            }
            #[doc(hidden)]
            unsafe fn __unique_ptr_get(repr: *mut c_void) -> *const CxxVector<Self> {
                extern "C" {
                    attr! {
                        #[link_name = concat!("cxxbridge1$unique_ptr$std$vector$", $segment, "$get")]
                        fn __unique_ptr_get(this: *const *mut c_void) -> *const CxxVector<$ty>;
                    }
                }
                __unique_ptr_get(&repr)
            }
            #[doc(hidden)]
            unsafe fn __unique_ptr_release(mut repr: *mut c_void) -> *mut CxxVector<Self> {
                extern "C" {
                    attr! {
                        #[link_name = concat!("cxxbridge1$unique_ptr$std$vector$", $segment, "$release")]
                        fn __unique_ptr_release(this: *mut *mut c_void) -> *mut CxxVector<$ty>;
                    }
                }
                __unique_ptr_release(&mut repr)
            }
            #[doc(hidden)]
            unsafe fn __unique_ptr_drop(mut repr: *mut c_void) {
                extern "C" {
                    attr! {
                        #[link_name = concat!("cxxbridge1$unique_ptr$std$vector$", $segment, "$drop")]
                        fn __unique_ptr_drop(this: *mut *mut c_void);
                    }
                }
                __unique_ptr_drop(&mut repr);
            }
        }
    };
}

macro_rules! impl_vector_element_for_primitive {
    ($ty:ident) => {
        impl_vector_element!(stringify!($ty), stringify!($ty), $ty);
    };
}

impl_vector_element_for_primitive!(u8);
impl_vector_element_for_primitive!(u16);
impl_vector_element_for_primitive!(u32);
impl_vector_element_for_primitive!(u64);
impl_vector_element_for_primitive!(usize);
impl_vector_element_for_primitive!(i8);
impl_vector_element_for_primitive!(i16);
impl_vector_element_for_primitive!(i32);
impl_vector_element_for_primitive!(i64);
impl_vector_element_for_primitive!(isize);
impl_vector_element_for_primitive!(f32);
impl_vector_element_for_primitive!(f64);

impl_vector_element!("string", "CxxString", CxxString);

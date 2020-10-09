use crate::cxx_string::CxxString;
use core::ffi::c_void;
use core::fmt::{self, Display};
use core::marker::PhantomData;
use core::mem;
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

    /// Returns a reference to an element without doing bounds checking.
    ///
    /// This is generally not recommended, use with caution! Calling this method
    /// with an out-of-bounds index is undefined behavior even if the resulting
    /// reference is not used.
    ///
    /// Matches the behavior of C++
    /// [std::vector\<T\>::operator\[\]][operator_at].
    ///
    /// [operator_at]: https://en.cppreference.com/w/cpp/container/vector/operator_at
    pub unsafe fn get_unchecked(&self, pos: usize) -> &T {
        &*T::__get_unchecked(self, pos)
    }

    /// Returns a slice to the underlying contiguous array of elements.
    pub fn as_slice(&self) -> &[T] {
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
            let ptr = unsafe { T::__get_unchecked(self, 0) };
            unsafe { slice::from_raw_parts(ptr, len) }
        }
    }
}

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
        Iter { v: self, index: 0 }
    }
}

impl<'a, T> Iterator for Iter<'a, T>
where
    T: VectorElement,
{
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.v.get(self.index);
        self.index += 1;
        next
    }
}

pub struct TypeName<T> {
    element: PhantomData<T>,
}

impl<T> TypeName<T> {
    pub const fn new() -> Self {
        TypeName {
            element: PhantomData,
        }
    }
}

impl<T> Display for TypeName<T>
where
    T: VectorElement,
{
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "CxxVector<{}>", T::__NAME)
    }
}

// Methods are private; not intended to be implemented outside of cxxbridge
// codebase.
#[doc(hidden)]
pub unsafe trait VectorElement: Sized {
    const __NAME: &'static dyn Display;
    fn __vector_size(v: &CxxVector<Self>) -> usize;
    unsafe fn __get_unchecked(v: &CxxVector<Self>, pos: usize) -> *const Self;
    fn __unique_ptr_null() -> *mut c_void;
    unsafe fn __unique_ptr_raw(raw: *mut CxxVector<Self>) -> *mut c_void;
    unsafe fn __unique_ptr_get(repr: *mut c_void) -> *const CxxVector<Self>;
    unsafe fn __unique_ptr_release(repr: *mut c_void) -> *mut CxxVector<Self>;
    unsafe fn __unique_ptr_drop(repr: *mut c_void);
}

macro_rules! impl_vector_element {
    ($segment:expr, $name:expr, $ty:ty) => {
        const_assert_eq!(1, mem::align_of::<CxxVector<$ty>>());

        unsafe impl VectorElement for $ty {
            const __NAME: &'static dyn Display = &$name;
            fn __vector_size(v: &CxxVector<$ty>) -> usize {
                extern "C" {
                    attr! {
                        #[link_name = concat!("cxxbridge05$std$vector$", $segment, "$size")]
                        fn __vector_size(_: &CxxVector<$ty>) -> usize;
                    }
                }
                unsafe { __vector_size(v) }
            }
            unsafe fn __get_unchecked(v: &CxxVector<$ty>, pos: usize) -> *const $ty {
                extern "C" {
                    attr! {
                        #[link_name = concat!("cxxbridge05$std$vector$", $segment, "$get_unchecked")]
                        fn __get_unchecked(_: &CxxVector<$ty>, _: usize) -> *const $ty;
                    }
                }
                __get_unchecked(v, pos)
            }
            fn __unique_ptr_null() -> *mut c_void {
                extern "C" {
                    attr! {
                        #[link_name = concat!("cxxbridge05$unique_ptr$std$vector$", $segment, "$null")]
                        fn __unique_ptr_null(this: *mut *mut c_void);
                    }
                }
                let mut repr = ptr::null_mut::<c_void>();
                unsafe { __unique_ptr_null(&mut repr) }
                repr
            }
            unsafe fn __unique_ptr_raw(raw: *mut CxxVector<Self>) -> *mut c_void {
                extern "C" {
                    attr! {
                        #[link_name = concat!("cxxbridge05$unique_ptr$std$vector$", $segment, "$raw")]
                        fn __unique_ptr_raw(this: *mut *mut c_void, raw: *mut CxxVector<$ty>);
                    }
                }
                let mut repr = ptr::null_mut::<c_void>();
                __unique_ptr_raw(&mut repr, raw);
                repr
            }
            unsafe fn __unique_ptr_get(repr: *mut c_void) -> *const CxxVector<Self> {
                extern "C" {
                    attr! {
                        #[link_name = concat!("cxxbridge05$unique_ptr$std$vector$", $segment, "$get")]
                        fn __unique_ptr_get(this: *const *mut c_void) -> *const CxxVector<$ty>;
                    }
                }
                __unique_ptr_get(&repr)
            }
            unsafe fn __unique_ptr_release(mut repr: *mut c_void) -> *mut CxxVector<Self> {
                extern "C" {
                    attr! {
                        #[link_name = concat!("cxxbridge05$unique_ptr$std$vector$", $segment, "$release")]
                        fn __unique_ptr_release(this: *mut *mut c_void) -> *mut CxxVector<$ty>;
                    }
                }
                __unique_ptr_release(&mut repr)
            }
            unsafe fn __unique_ptr_drop(mut repr: *mut c_void) {
                extern "C" {
                    attr! {
                        #[link_name = concat!("cxxbridge05$unique_ptr$std$vector$", $segment, "$drop")]
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

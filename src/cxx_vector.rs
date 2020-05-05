use std::ffi::c_void;
use std::fmt::{self, Display};
use std::marker::PhantomData;
use std::mem;
use std::ptr;

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
            Some(unsafe { T::__get_unchecked(self, pos) })
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
        T::__get_unchecked(self, pos)
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
    unsafe fn __get_unchecked(v: &CxxVector<Self>, pos: usize) -> &Self;
    fn __unique_ptr_null() -> *mut c_void;
    unsafe fn __unique_ptr_raw(raw: *mut CxxVector<Self>) -> *mut c_void;
    unsafe fn __unique_ptr_get(repr: *mut c_void) -> *const CxxVector<Self>;
    unsafe fn __unique_ptr_release(repr: *mut c_void) -> *mut CxxVector<Self>;
    unsafe fn __unique_ptr_drop(repr: *mut c_void);
}

macro_rules! impl_vector_element_for_primitive {
    ($ty:ident) => {
        const_assert_eq!(1, mem::align_of::<CxxVector<$ty>>());

        unsafe impl VectorElement for $ty {
            const __NAME: &'static dyn Display = &stringify!($ty);
            fn __vector_size(v: &CxxVector<$ty>) -> usize {
                extern "C" {
                    attr! {
                        #[link_name = concat!("cxxbridge03$std$vector$", stringify!($ty), "$size")]
                        fn __vector_size(_: &CxxVector<$ty>) -> usize;
                    }
                }
                unsafe { __vector_size(v) }
            }
            unsafe fn __get_unchecked(v: &CxxVector<$ty>, pos: usize) -> &$ty {
                extern "C" {
                    attr! {
                        #[link_name = concat!("cxxbridge03$std$vector$", stringify!($ty), "$get_unchecked")]
                        fn __get_unchecked(_: &CxxVector<$ty>, _: usize) -> *const $ty;
                    }
                }
                &*__get_unchecked(v, pos)
            }
            fn __unique_ptr_null() -> *mut c_void {
                extern "C" {
                    attr! {
                        #[link_name = concat!("cxxbridge03$unique_ptr$std$vector$", stringify!($ty), "$null")]
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
                        #[link_name = concat!("cxxbridge03$unique_ptr$std$vector$", stringify!($ty), "$raw")]
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
                        #[link_name = concat!("cxxbridge03$unique_ptr$std$vector$", stringify!($ty), "$get")]
                        fn __unique_ptr_get(this: *const *mut c_void) -> *const CxxVector<$ty>;
                    }
                }
                __unique_ptr_get(&repr)
            }
            unsafe fn __unique_ptr_release(mut repr: *mut c_void) -> *mut CxxVector<Self> {
                extern "C" {
                    attr! {
                        #[link_name = concat!("cxxbridge03$unique_ptr$std$vector$", stringify!($ty), "$release")]
                        fn __unique_ptr_release(this: *mut *mut c_void) -> *mut CxxVector<$ty>;
                    }
                }
                __unique_ptr_release(&mut repr)
            }
            unsafe fn __unique_ptr_drop(mut repr: *mut c_void) {
                extern "C" {
                    attr! {
                        #[link_name = concat!("cxxbridge03$unique_ptr$std$vector$", stringify!($ty), "$drop")]
                        fn __unique_ptr_drop(this: *mut *mut c_void);
                    }
                }
                __unique_ptr_drop(&mut repr);
            }
        }
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

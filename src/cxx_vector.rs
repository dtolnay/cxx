use std::mem;

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

    /// Appends an element to the back of the vector.
    pub fn push_back(&mut self, item: &T) {
        T::__push_back(self, item);
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
        self.index = self.index + 1;
        self.v.get(self.index - 1)
    }
}

// Methods are private; not intended to be implemented outside of cxxbridge
// codebase.
#[doc(hidden)]
pub unsafe trait VectorElement: Sized {
    fn __vector_size(v: &CxxVector<Self>) -> usize;
    unsafe fn __get_unchecked(v: &CxxVector<Self>, pos: usize) -> &Self;
    fn __push_back(v: &CxxVector<Self>, item: &Self);
}

macro_rules! impl_vector_element_for_primitive {
    ($ty:ident) => {
        unsafe impl VectorElement for $ty {
            fn __vector_size(v: &CxxVector<$ty>) -> usize {
                extern "C" {
                    attr! {
                        #[link_name = concat!("cxxbridge02$std$vector$", stringify!($ty), "$size")]
                        fn __vector_size(_: &CxxVector<$ty>) -> usize;
                    }
                }
                unsafe { __vector_size(v) }
            }
            unsafe fn __get_unchecked(v: &CxxVector<$ty>, pos: usize) -> &$ty {
                extern "C" {
                    attr! {
                        #[link_name = concat!("cxxbridge02$std$vector$", stringify!($ty), "$get_unchecked")]
                        fn __get_unchecked(_: &CxxVector<$ty>, _: usize) -> *const $ty;
                    }
                }
                &*__get_unchecked(v, pos)
            }
            fn __push_back(v: &CxxVector<$ty>, item: &$ty) {
                extern "C" {
                    attr! {
                        #[link_name = concat!("cxxbridge02$std$vector$", stringify!($ty), "$push_back")]
                        fn __push_back(_: &CxxVector<$ty>, _: &$ty);
                    }
                }
                unsafe { __push_back(v, item) }
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

const_assert_eq!(1, mem::align_of::<CxxVector<usize>>());

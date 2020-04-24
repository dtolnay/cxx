use std::mem;

/// Binding to C++ `std::vector<T>`.
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

impl<T: VectorElement> CxxVector<T> {
    /// Returns the number of elements in the vector.
    pub fn len(&self) -> usize {
        T::__vector_size(self)
    }

    /// Returns true if the vector contains no elements.
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

impl<'a, T: VectorElement> IntoIterator for &'a CxxVector<T> {
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        Iter { v: self, index: 0 }
    }
}

impl<'a, T: VectorElement> Iterator for Iter<'a, T> {
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
    unsafe fn __get_unchecked(v: &CxxVector<Self>, pos: usize) -> &Self;
    fn __vector_size(v: &CxxVector<Self>) -> usize;
    fn __push_back(v: &CxxVector<Self>, item: &Self);
}

cxxbridge_macro::vector_builtin!(u8);
cxxbridge_macro::vector_builtin!(u16);
cxxbridge_macro::vector_builtin!(u32);
cxxbridge_macro::vector_builtin!(u64);
cxxbridge_macro::vector_builtin!(usize);
cxxbridge_macro::vector_builtin!(i8);
cxxbridge_macro::vector_builtin!(i16);
cxxbridge_macro::vector_builtin!(i32);
cxxbridge_macro::vector_builtin!(i64);
cxxbridge_macro::vector_builtin!(isize);
cxxbridge_macro::vector_builtin!(f32);
cxxbridge_macro::vector_builtin!(f64);

const_assert_eq!(1, mem::align_of::<CxxVector<usize>>());

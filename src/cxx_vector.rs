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

impl<T: VectorElement<T>> CxxVector<T> {
    /// Returns the length of the vector in bytes.
    pub fn size(&self) -> usize {
        T::__vector_length(self)
    }

    pub fn get_unchecked(&self, pos: usize) -> &T {
        T::__get_unchecked(self, pos)
    }

    /// Returns true if `self` has a length of zero bytes.
    pub fn is_empty(&self) -> bool {
        self.size() == 0
    }

    pub fn get(&self, pos: usize) -> Option<&T> {
        if pos < self.size() {
            Some(self.get_unchecked(pos))
        } else {
            None
        }
    }

    pub fn push_back(&mut self, item: &T) {
        T::__push_back(self, item);
    }
}

pub struct VectorIntoIterator<'a, T> {
    v: &'a CxxVector<T>,
    index: usize,
}

impl<'a, T: VectorElement<T>> IntoIterator for &'a CxxVector<T> {
    type Item = &'a T;
    type IntoIter = VectorIntoIterator<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        VectorIntoIterator { v: self, index: 0 }
    }
}

impl<'a, T: VectorElement<T>> Iterator for VectorIntoIterator<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        self.index = self.index + 1;
        self.v.get(self.index - 1)
    }
}

// Methods are private; not intended to be implemented outside of cxxbridge
// codebase.
#[doc(hidden)]
pub unsafe trait VectorElement<T> {
    fn __get_unchecked(v: &CxxVector<T>, pos: usize) -> &T;
    fn __vector_length(v: &CxxVector<T>) -> usize;
    fn __push_back(v: &CxxVector<T>, item: &T);
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

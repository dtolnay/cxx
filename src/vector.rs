pub trait VectorTarget<T> {
    fn get_unchecked(v: &RealVector<T>, pos: usize) -> &T
    where
        Self: Sized;
    fn vector_length(v: &RealVector<T>) -> usize
    where
        Self: Sized;
    fn push_back(v: &RealVector<T>, item: &T)
    where
        Self: Sized;
}

/// Binding to C++ `std::vector`.
///
/// # Invariants
///
/// As an invariant of this API and the static analysis of the cxx::bridge
/// macro, in Rust code we can never obtain a `Vector` by value. C++'s vector
/// requires a move constructor and may hold internal pointers, which is not
/// compatible with Rust's move behavior. Instead in Rust code we will only ever
/// look at a Vector through a reference or smart pointer, as in `&Vector`
/// or `UniquePtr<Vector>`.
#[repr(C)]
pub struct RealVector<T> {
    _private: [T; 0],
}

impl<T: VectorTarget<T>> RealVector<T> {
    /// Returns the length of the vector in bytes.
    pub fn size(&self) -> usize {
        T::vector_length(self)
    }

    pub fn get_unchecked(&self, pos: usize) -> &T {
        T::get_unchecked(self, pos)
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
        T::push_back(self, item);
    }
}

unsafe impl<T> Send for RealVector<T> where T: Send + VectorTarget<T> {}

pub struct VectorIntoIterator<'a, T> {
    v: &'a RealVector<T>,
    index: usize,
}

impl<'a, T: VectorTarget<T>> IntoIterator for &'a RealVector<T> {
    type Item = &'a T;
    type IntoIter = VectorIntoIterator<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        VectorIntoIterator { v: self, index: 0 }
    }
}

impl<'a, T: VectorTarget<T>> Iterator for VectorIntoIterator<'a, T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<Self::Item> {
        self.index = self.index + 1;
        self.v.get(self.index - 1)
    }
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

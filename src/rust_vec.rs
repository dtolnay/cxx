use std::mem;

#[repr(C)]
pub struct RustVec<T> {
    repr: Vec<T>,
}

impl<T> RustVec<T> {
    pub fn from(v: Vec<T>) -> Self {
        RustVec { repr: v }
    }

    pub fn from_ref(v: &Vec<T>) -> &Self {
        unsafe { &*(v as *const Vec<T> as *const RustVec<T>) }
    }

    pub fn into_vec(self) -> Vec<T> {
        self.repr
    }

    pub fn as_vec(&self) -> &Vec<T> {
        &self.repr
    }

    pub fn as_mut_vec(&mut self) -> &mut Vec<T> {
        &mut self.repr
    }

    pub fn len(&self) -> usize {
        self.repr.len()
    }

    pub fn as_ptr(&self) -> *const T {
        self.repr.as_ptr()
    }
}

const_assert_eq!(mem::size_of::<[usize; 3]>(), mem::size_of::<Vec<i32>>());
const_assert_eq!(mem::align_of::<usize>(), mem::align_of::<Vec<i32>>());

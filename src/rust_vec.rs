use crate::rust_string::RustString;
use alloc::string::String;
use alloc::vec::Vec;
use core::mem::ManuallyDrop;

#[repr(C)]
pub struct RustVec<T> {
    pub(crate) repr: Vec<T>,
}

impl<T> RustVec<T> {
    pub fn new() -> Self {
        RustVec { repr: Vec::new() }
    }

    pub fn from(v: Vec<T>) -> Self {
        RustVec { repr: v }
    }

    pub fn from_ref(v: &Vec<T>) -> &Self {
        unsafe { &*(v as *const Vec<T> as *const RustVec<T>) }
    }

    pub fn from_mut(v: &mut Vec<T>) -> &mut Self {
        unsafe { &mut *(v as *mut Vec<T> as *mut RustVec<T>) }
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

    pub fn capacity(&self) -> usize {
        self.repr.capacity()
    }

    pub fn as_ptr(&self) -> *const T {
        self.repr.as_ptr()
    }

    pub fn reserve_total(&mut self, cap: usize) {
        let len = self.repr.len();
        if cap > len {
            self.repr.reserve(cap - len);
        }
    }

    pub unsafe fn set_len(&mut self, len: usize) {
        self.repr.set_len(len);
    }
}

impl RustVec<RustString> {
    pub fn from_vec_string(v: Vec<String>) -> Self {
        let mut v = ManuallyDrop::new(v);
        let ptr = v.as_mut_ptr().cast::<RustString>();
        let len = v.len();
        let cap = v.capacity();
        Self::from(unsafe { Vec::from_raw_parts(ptr, len, cap) })
    }

    pub fn from_ref_vec_string(v: &Vec<String>) -> &Self {
        Self::from_ref(unsafe { &*(v as *const Vec<String> as *const Vec<RustString>) })
    }

    pub fn from_mut_vec_string(v: &mut Vec<String>) -> &mut Self {
        Self::from_mut(unsafe { &mut *(v as *mut Vec<String> as *mut Vec<RustString>) })
    }

    pub fn into_vec_string(self) -> Vec<String> {
        let mut v = ManuallyDrop::new(self.repr);
        let ptr = v.as_mut_ptr().cast::<String>();
        let len = v.len();
        let cap = v.capacity();
        unsafe { Vec::from_raw_parts(ptr, len, cap) }
    }

    pub fn as_vec_string(&self) -> &Vec<String> {
        unsafe { &*(&self.repr as *const Vec<RustString> as *const Vec<String>) }
    }

    pub fn as_mut_vec_string(&mut self) -> &mut Vec<String> {
        unsafe { &mut *(&mut self.repr as *mut Vec<RustString> as *mut Vec<String>) }
    }
}

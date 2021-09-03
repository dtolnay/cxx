#![allow(missing_docs)]

use crate::rust_string::RustString;
use alloc::string::String;
use alloc::vec::Vec;
use core::ffi::c_void;
use core::marker::PhantomData;
use core::mem::{self, ManuallyDrop, MaybeUninit};
use core::ptr;

// ABI compatible with C++ rust::Vec<T> (not necessarily alloc::vec::Vec<T>).
#[repr(C)]
pub struct RustVec<T> {
    repr: [MaybeUninit<usize>; mem::size_of::<Vec<c_void>>() / mem::size_of::<usize>()],
    marker: PhantomData<Vec<T>>,
}

impl<T> RustVec<T> {
    pub fn new() -> Self {
        Self::from(Vec::new())
    }

    pub fn from(v: Vec<T>) -> Self {
        unsafe { mem::transmute::<Vec<T>, RustVec<T>>(v) }
    }

    pub fn from_ref(v: &Vec<T>) -> &Self {
        unsafe { &*(v as *const Vec<T> as *const RustVec<T>) }
    }

    pub fn from_mut(v: &mut Vec<T>) -> &mut Self {
        unsafe { &mut *(v as *mut Vec<T> as *mut RustVec<T>) }
    }

    pub fn into_vec(self) -> Vec<T> {
        unsafe { mem::transmute::<RustVec<T>, Vec<T>>(self) }
    }

    pub fn as_vec(&self) -> &Vec<T> {
        unsafe { &*(self as *const RustVec<T> as *const Vec<T>) }
    }

    pub fn as_mut_vec(&mut self) -> &mut Vec<T> {
        unsafe { &mut *(self as *mut RustVec<T> as *mut Vec<T>) }
    }

    pub fn len(&self) -> usize {
        self.as_vec().len()
    }

    pub fn capacity(&self) -> usize {
        self.as_vec().capacity()
    }

    pub fn as_ptr(&self) -> *const T {
        self.as_vec().as_ptr()
    }

    pub fn reserve_total(&mut self, new_cap: usize) {
        let vec = self.as_mut_vec();
        if new_cap > vec.capacity() {
            let additional = new_cap - vec.len();
            vec.reserve(additional);
        }
    }

    pub unsafe fn set_len(&mut self, len: usize) {
        unsafe { self.as_mut_vec().set_len(len) }
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
        let mut v = ManuallyDrop::new(self.into_vec());
        let ptr = v.as_mut_ptr().cast::<String>();
        let len = v.len();
        let cap = v.capacity();
        unsafe { Vec::from_raw_parts(ptr, len, cap) }
    }

    pub fn as_vec_string(&self) -> &Vec<String> {
        unsafe { &*(self as *const RustVec<RustString> as *const Vec<String>) }
    }

    pub fn as_mut_vec_string(&mut self) -> &mut Vec<String> {
        unsafe { &mut *(self as *mut RustVec<RustString> as *mut Vec<String>) }
    }
}

impl<T> Drop for RustVec<T> {
    fn drop(&mut self) {
        unsafe { ptr::drop_in_place(self.as_mut_vec()) }
    }
}

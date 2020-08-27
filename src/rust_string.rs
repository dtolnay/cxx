use std::mem;

#[repr(C)]
pub struct RustString {
    repr: String,
}

impl RustString {
    pub fn from(s: String) -> Self {
        RustString { repr: s }
    }

    pub fn from_ref(s: &String) -> &Self {
        unsafe { &*(s as *const String as *const RustString) }
    }

    pub fn from_mut(s: &mut String) -> &mut Self {
        unsafe { &mut *(s as *mut String as *mut RustString) }
    }

    pub fn into_string(self) -> String {
        self.repr
    }

    pub fn as_string(&self) -> &String {
        &self.repr
    }

    pub fn as_mut_string(&mut self) -> &mut String {
        &mut self.repr
    }
}

const_assert_eq!(mem::size_of::<[usize; 3]>(), mem::size_of::<String>());
const_assert_eq!(mem::align_of::<usize>(), mem::align_of::<String>());

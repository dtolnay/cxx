use cxx::ExternType;
use std::ops::Deref;

#[repr(C)]
pub struct ScopedPtr<T: ScopedPtrTarget> {
    ptr: *mut T,
    padding: [*const std::ffi::c_void; 2],
}

pub trait ScopedPtrTarget: Sized {
    unsafe fn drop_in_place(ptr: &mut ScopedPtr<Self>);
}

impl<T: ScopedPtrTarget> Deref for ScopedPtr<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.ptr }
    }
}

impl<T: ScopedPtrTarget> Drop for ScopedPtr<T> {
    fn drop(&mut self) {
        unsafe { T::drop_in_place(self) }
    }
}

unsafe impl ExternType for ScopedPtr<ffi::Class> {
    type Id = cxx::type_id!("ScopedClass");
    type Kind = cxx::kind::Trivial;
}

impl ScopedPtrTarget for ffi::Class {
    unsafe fn drop_in_place(ptr: &mut ScopedPtr<Self>) {
        ptr.drop_in_place();
    }
}

#[cxx::bridge]
mod ffi {
    unsafe extern "C++" {
        include!("scoped-ptr-demo/include/demo.h");

        type ScopedClass = crate::ScopedPtr<Class>;
        unsafe fn drop_in_place(self: &mut ScopedClass);

        fn getclass() -> ScopedClass;

        type Class;
        fn print(self: &Class);
    }
}

fn main() {
    ffi::getclass().print();
}

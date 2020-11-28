use cxx::ExternType;
use std::ops::Deref;

#[repr(transparent)]
pub struct ScopedPtr<T>(*mut T);

impl<T> Deref for ScopedPtr<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.0 }
    }
}

unsafe impl ExternType for ScopedPtr<ffi::Class> {
    type Id = cxx::type_id!("ScopedClass");
    type Kind = cxx::kind::Opaque;
}

#[cxx::bridge]
mod ffi {
    unsafe extern "C++" {
        include!("scoped-ptr-demo/include/demo.h");

        type ScopedClass = crate::ScopedPtr<Class>;

        fn run();

        type Class;
        fn print(self: &Class);
    }

    extern "Rust" {
        fn recv(a: &ScopedClass, b: &ScopedClass);
    }
}

fn recv(a: &ffi::ScopedClass, b: &ffi::ScopedClass) {
    a.print();
    b.print();
}

fn main() {
    ffi::run();
}

use cxx::ExternType;
use std::marker::PhantomPinned;

struct Opaque(PhantomPinned);

unsafe impl ExternType for Opaque {
    type Id = cxx::type_id!("Opaque");
    type Kind = cxx::kind::Opaque;
}

#[cxx::bridge]
mod ffi {
    unsafe extern "C++" {
        type Opaque = crate::Opaque;
        fn f(arg: &mut Opaque);
        fn g(&mut self);
        fn h(self: &mut Opaque);
    }
}

fn main() {}

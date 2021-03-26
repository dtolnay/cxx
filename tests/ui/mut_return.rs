#[cxx::bridge]
mod ffi {
    unsafe extern "C++" {
        type Thing;

        fn f(t: &Thing) -> Pin<&mut CxxString>;
        unsafe fn g(t: &Thing) -> Pin<&mut CxxString>;
    }
}

fn main() {}

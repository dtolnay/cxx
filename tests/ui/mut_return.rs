#[cxx::bridge]
mod ffi {
    extern "Rust" {
        type Mut<'a>;
    }

    unsafe extern "C++" {
        type Thing;

        fn f<'a>(t: &'a Thing) -> Pin<&'a mut CxxString>;
        unsafe fn g<'a>(t: &'a Thing) -> Pin<&'a mut CxxString>;
        fn h<'a>(t: Box<Mut>) -> Pin<&'a mut CxxString>;
        fn i<'a>(t: Box<Mut<'a>>) -> Pin<&'a mut CxxString>;
        fn j<'a>(t: &'a Thing) -> &'a mut [u8];
    }
}

fn main() {}

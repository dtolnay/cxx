#[cxx::bridge]
mod ffi {
    unsafe extern "C++" {
        type C;

        fn not_unsafe_ptr(c: *mut C);
    }
}

#[cxx::bridge]
mod ffi2 {
    unsafe extern "C++" {
        type C;

        fn get_neither_const_nor_mut() -> *C;
    }
}

#[cxx::bridge]
mod ffi3 {
    unsafe extern "C++" {
        type C;

        fn get_ptr_ptr() -> *mut *mut C;
        fn get_ptr_reference() -> *mut & C;
        fn get_reference_ptr() -> & *mut C;
    }
}

fn main() {}

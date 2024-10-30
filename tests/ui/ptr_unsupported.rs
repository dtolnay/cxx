#[cxx::bridge]
mod ffi {
    unsafe extern "C++" {
        type C;

        fn get_ptr_to_reference() -> *mut &C;
        fn get_uniqueptr_to_ptr() -> UniquePtr<*mut C>;
    }
}

fn main() {}

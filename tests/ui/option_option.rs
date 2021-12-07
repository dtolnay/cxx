#[cxx::bridge]
mod ffi {
    extern "C++" {
        unsafe fn f(_: Option<Option<&u32>>);
    }
}

fn main() {}

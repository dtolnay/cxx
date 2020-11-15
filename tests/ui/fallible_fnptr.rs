#[cxx::bridge]
mod ffi {
    extern "C++" {
        fn f(callback: fn() -> Result<()>);
    }
}

fn main() {}

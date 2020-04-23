#[cxx::bridge]
mod ffi {
    extern "C" {
        fn f(self: &Unrecognized);
    }
}

fn main() {}

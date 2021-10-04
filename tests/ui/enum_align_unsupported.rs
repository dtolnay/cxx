#[cxx::bridge]
mod ffi {
    #[repr(align(2))]
    enum Bad {
        A,
    }
}

fn main() {}

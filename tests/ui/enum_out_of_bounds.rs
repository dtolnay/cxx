#[cxx::bridge]
mod ffi {
    #[repr(u32)]
    enum Bad {
        A = 0xFFFF_FFFF_FFFF_FFFF,
    }
}

fn main() {}

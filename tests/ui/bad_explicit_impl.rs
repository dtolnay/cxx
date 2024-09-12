#[cxx::bridge]
mod ffi {
    struct S {
        x: u8,
    }

    impl<'a> fn() -> &'a S {}
}

fn main() {}

#[cxx::bridge]
mod ffi {
    enum Bad {
        A(i32),
        B = 1,
    }
}

fn main() {}

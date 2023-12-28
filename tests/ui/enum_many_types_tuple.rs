#[cxx::bridge]
mod ffi {
    enum Bad {
        A(i32, bool),
        B(i32),
    }
}

fn main() {}

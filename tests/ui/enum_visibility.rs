#[cxx::bridge]
mod ffi {
    enum Bad {
        A(pub i32),
        B(bool),
    }
}

fn main() {}

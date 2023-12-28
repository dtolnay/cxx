#[cxx::bridge]
mod ffi {
    enum Bad {
        A(pub i32),
        B = 1,
    }
}

fn main() {}
#[cxx::bridge]
mod ffi {
    enum Bad {
        A{age: i32},
        B(i32),
    }
}

fn main() {}

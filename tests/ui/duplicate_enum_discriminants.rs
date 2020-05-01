#[cxx::bridge]
mod ffi {
    enum A {
        V1 = 10,
        V2 = 10,
    }

    enum B {
        V1 = 10,
        V2,
        V3 = 11,
    }
}

fn main() {}

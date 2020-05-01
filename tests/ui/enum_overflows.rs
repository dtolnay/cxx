#[cxx::bridge]
mod ffi {
    enum Good1 {
        A = 0xffffffff,
    }
    enum Good2 {
        B = 0xffffffff,
        C = 2020,
    }
    enum Bad {
        D = 0xfffffffe,
        E,
        F,
    }
}

fn main() {}

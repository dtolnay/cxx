#[cxx::bridge]
mod ffi {
    #[cxx_name = "operator=="]
    struct S {
        #[cxx_name = "operator<"]
        field: usize,
    }

    #[cxx_name = "operator>"]
    enum E {
        Variant1,
        #[cxx_name = "operator!"]
        Variant2,
        Variant3,
    }

    unsafe extern "C++" {
        #[cxx_name = "operator+"]
        type C2;

        #[cxx_name = "operator*"]
        type Alias = some_other_crate::SomeOtherType;
    }

    extern "Rust" {
        #[cxx_name = "operator/"]
        type R2;
    }
}

fn main() {}

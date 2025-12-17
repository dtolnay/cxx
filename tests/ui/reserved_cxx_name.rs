#[cxx::bridge]
mod ffi {
    #[cxx_name = "constinit"]
    struct S {
        consteval: usize,

        #[cxx_name = "constexpr"]
        field2: usize,
    }

    #[cxx_name = "bitand"]
    enum E {
        Variant1,
        bitor,
        Variant3,
    }

    unsafe extern "C++" {
        fn const_cast();

        type C;
        fn reinterpret_cast(self: &C);

        #[cxx_name = "static_cast"]
        type C2;

        #[cxx_name = "dynamic_cast"]
        type Alias = some_other_crate::SomeOtherType;
    }

    extern "Rust" {
        fn private();

        type R;
        fn protected(self: &R);

        #[cxx_name = "public"]
        type R2;
    }
}

fn main() {}

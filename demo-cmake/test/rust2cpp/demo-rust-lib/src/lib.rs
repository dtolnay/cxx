#[cxx::bridge(namespace = org::example)]
mod ffi {
    extern "C" {
        include!("demo.h");

        type ThingC;
    }

    extern "Rust" {
        fn print_str(s: &CxxString);
    }
}


fn print_str(s: &cxx::CxxString) {
    println!("called back with s={}", s);
}

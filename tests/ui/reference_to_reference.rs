#[cxx::bridge]
mod ffi {
    unsafe extern "C++" {
        type ThingC;
        fn ref_c(t: &ThingC);
        fn rvalue_c(t: &&ThingC);
        fn repro_c(t: &&&ThingC);
    }
    extern "Rust" {
        type ThingR;
        fn ref_r(t: &ThingC);
        fn rvalue_r(t: &ThingC);
        fn repro_r(t: &&&ThingR);
    }
}

fn main() {}

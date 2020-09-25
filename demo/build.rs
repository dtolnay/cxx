fn main() {
    cxx_build::bridge("src/main.rs")
        .file("src/demo.cc")
        .flag_if_supported(cxxbridge_flags::STD)
        .compile("cxxbridge-demo");

    println!("cargo:rerun-if-changed=src/main.rs");
    println!("cargo:rerun-if-changed=src/demo.cc");
    println!("cargo:rerun-if-changed=include/demo.h");
}

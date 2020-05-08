fn main() {
    cxx_build::bridge("src/main.rs")
        .file("../demo-cxx/demo.cc")
        .cpp(true)
        .flag("-std=c++11")
        .compile("cxxbridge-demo");

    println!("cargo:rerun-if-changed=src/main.rs");
    println!("cargo:rerun-if-changed=../demo-cxx/demo.h");
    println!("cargo:rerun-if-changed=../demo-cxx/demo.cc");
}

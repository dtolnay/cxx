fn main() {
    cc::Build::new()
        .file("src/cxx.cc")
        .cpp(true)
        .cpp_link_stdlib(None) // linked via link-cplusplus crate
        .flag("-std=c++11")
        .compile("cxxbridge03");
    println!("cargo:rerun-if-changed=src/cxx.cc");
    println!("cargo:rerun-if-changed=include/cxx.h");
}

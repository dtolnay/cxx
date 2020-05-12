fn main() {
    cc::Build::new()
        .file("src/cxx.cc")
        .cpp(true)
        .cpp_link_stdlib(None) // linked via link-cplusplus crate
        .flag_if_supported(if cfg!(feature = "c++17") {
            "-std=c++17"
        } else if cfg!(feature = "c++14") {
            "-std=c++14"
        } else {
            "-std=c++11"
        })
        .compile("cxxbridge03");
    println!("cargo:rerun-if-changed=src/cxx.cc");
    println!("cargo:rerun-if-changed=include/cxx.h");
}

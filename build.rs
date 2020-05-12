fn main() {
    let flag_prefix = if cfg!(target_env = "msvc") {
        "/std:"
    } else {
        "-std="
    };
    let cpp_version = if cfg!(feature = "c++14") {
        "14"
    } else if cfg!(feature = "c++17") {
        "17"
    } else {
        "11"
    };
    cc::Build::new()
        .file("src/cxx.cc")
        .cpp(true)
        .cpp_link_stdlib(None) // linked via link-cplusplus crate
        .flag_if_supported(&format!("{}c++{}", flag_prefix, cpp_version))
        .compile("cxxbridge03");
    println!("cargo:rerun-if-changed=src/cxx.cc");
    println!("cargo:rerun-if-changed=include/cxx.h");
}

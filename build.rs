fn main() {
    cc::Build::new()
        .file("src/cxx.cc")
        .flag("-std=c++11")
        .compile("cxxbridge02");
    println!("cargo:rerun-if-changed=src/cxx.cc");
    println!("cargo:rerun-if-changed=include/cxx.h");
}

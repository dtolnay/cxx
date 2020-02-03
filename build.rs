fn main() {
    cc::Build::new()
        .file("src/cxxbridge.cc")
        .flag("-std=c++11")
        .compile("cxxbridge01");
    println!("cargo:rerun-if-changed=src/cxxbridge.cc");
    println!("cargo:rerun-if-changed=include/cxxbridge.h");
}

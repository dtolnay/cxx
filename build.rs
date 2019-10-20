fn main() {
    cc::Build::new()
        .file("src/cxxbridge.cc")
        .flag("-std=c++11")
        .compile("cxxbridge00");
    println!("cargo:rustc-flags=-l dylib=stdc++");
    println!("cargo:rerun-if-changed=src/cxxbridge.cc");
    println!("cargo:rerun-if-changed=include/cxxbridge.h");
}

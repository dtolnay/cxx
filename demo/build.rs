fn main() {
    cxx_build::CFG.change_detection = true;
    cxx_build::bridge("src/main.rs")
        .file("src/blobstore.cc")
        .std("c++14")
        .compile("cxxbridge-demo");

    println!("cargo:rerun-if-changed=src/blobstore.cc");
    println!("cargo:rerun-if-changed=include/blobstore.h");
}

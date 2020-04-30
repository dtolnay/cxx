fn main() {
    if cfg!(trybuild) {
        return;
    }

    cxx_build::bridge("lib.rs")
        .file("tests.cc")
        .flag("-std=c++11")
        .compile("cxx-test-suite");
}

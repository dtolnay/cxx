fn main() {
    if cfg!(trybuild) {
        return;
    }

    cxx::Build::new()
        .bridge("lib.rs")
        .file("tests.cc")
        .flag("-std=c++11")
        .compile("cxx-test-suite");
}

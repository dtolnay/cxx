fn main() {
    if cfg!(trybuild) {
        return;
    }

    let sources = vec!["lib.rs", "module.rs"];
    cxx_build::bridges(sources)
        .file("tests.cc")
        .flag("-std=c++11")
        .compile("cxx-test-suite");
}

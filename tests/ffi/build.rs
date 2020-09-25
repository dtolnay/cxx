fn main() {
    if cfg!(trybuild) {
        return;
    }

    let sources = vec!["lib.rs", "module.rs"];
    cxx_build::bridges(sources)
        .file("tests.cc")
        .compile("cxx-test-suite");
}

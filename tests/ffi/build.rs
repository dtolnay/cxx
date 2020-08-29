fn main() {
    if cfg!(trybuild) {
        return;
    }

    let sources = vec!["lib.rs", "module.rs"];
    cxx_build::bridges(sources)
        .file("tests.cc")
        .flag_if_supported(cxxbridge_flags::STD)
        .compile("cxx-test-suite");
}

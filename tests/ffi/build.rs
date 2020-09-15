fn main() {
    if cfg!(trybuild) {
        return;
    }

    let sources = vec!["alias.rs", "alias2.rs", "lib.rs"];
    cxx_build::bridges(sources)
        .file("tests.cc")
        .flag_if_supported(cxxbridge_flags::STD)
        .compile("cxx-test-suite");
}

use cxx_build::CFG;

fn main() {
    if cfg!(trybuild) {
        return;
    }

    CFG.include_prefix = "tests/ffi";
    let sources = vec!["lib.rs", "extra.rs", "module.rs"];
    cxx_build::bridges(sources)
        .file("tests.cc")
        .flag_if_supported(cxxbridge_flags::STD)
        .compile("cxx-test-suite");
}

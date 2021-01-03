use cxx_build::CFG;

fn main() {
    if cfg!(trybuild) {
        return;
    }

    CFG.include_prefix = "tests/ffi";
    let sources = vec!["lib.rs", "module.rs"];
    let mut build = cxx_build::bridges(sources);
    build.file("tests.cc");
    build.flag_if_supported(cxxbridge_flags::STD);
    if cfg!(not(target_env = "msvc")) {
        build.define("CXX_TEST_INSTANTIATIONS", None);
    }
    build.compile("cxx-test-suite");
}

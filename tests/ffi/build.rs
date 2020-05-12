fn main() {
    if cfg!(trybuild) {
        return;
    }

    let flag_prefix = if cfg!(target_env = "msvc") {
        "/std:"
    } else {
        "-std="
    };
    let cpp_version = if cfg!(feature = "c++14") {
        "14"
    } else if cfg!(feature = "c++17") {
        "17"
    } else {
        "11"
    };
    let sources = vec!["lib.rs", "module.rs"];
    dbg!(format!("{}c++{}", flag_prefix, cpp_version));
    cxx_build::bridges(sources)
        .file("tests.cc")
        .flag_if_supported(&format!("{}c++{}", flag_prefix, cpp_version))
        .compile("cxx-test-suite");
}
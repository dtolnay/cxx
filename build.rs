use std::env;
use std::path::Path;

fn main() {
    cc::Build::new()
        .file("src/cxx.cc")
        .cpp(true)
        .cpp_link_stdlib(None) // linked via link-cplusplus crate
        .flag_if_supported(cxxbridge_flags::STD)
        .compile("cxxbridge05");
    println!("cargo:rerun-if-changed=src/cxx.cc");
    println!("cargo:rerun-if-changed=include/cxx.h");
    println!("cargo:rustc-cfg=built_with_cargo");
    if let Some(manifest_dir) = env::var_os("CARGO_MANIFEST_DIR") {
        let cxx_h = Path::new(&manifest_dir).join("include").join("cxx.h");
        println!("cargo:HEADER={}", cxx_h.to_string_lossy());
    }
}

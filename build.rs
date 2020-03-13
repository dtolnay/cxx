use std::path::Path;

fn main() {
    build();

    if probe("shared_ptr.cc") {
        println!("cargo:rustc-cfg=shared_ptr");
    }
}

fn build() {
    println!("cargo:rerun-if-changed=src/cxx.cc");
    println!("cargo:rerun-if-changed=include/cxx.h");

    cc::Build::new()
        .file("src/cxx.cc")
        .flag("-std=c++11")
        .compile("cxxbridge02");
}

fn probe(which: &str) -> bool {
    let path = Path::new("probe").join(which);
    assert!(path.exists());

    let mut cmd = match cc::Build::new()
        .flag("-std=c++11")
        .try_get_compiler()
    {
        Ok(tool) => tool.to_command(),
        Err(_) => return false,
    };

    println!("cargo:rerun-if-changed=probe/{}", which);

    match cmd.arg("-S").arg(path).arg("-o").arg("/dev/null").status() {
        Ok(status) => status.success(),
        Err(_) => false,
    }
}

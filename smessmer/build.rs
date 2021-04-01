fn main() {
    cxx_build::bridge("src/main.rs")
        .file("src/roundtrip.cc")
        .compile("smessmer");
}

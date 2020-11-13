fn main() {
    cxx_build::bridges(&["src/main.rs", "src/handle.rs"])
        .file("src/test.cc")
        .compile("cxx-handles-demo");
}

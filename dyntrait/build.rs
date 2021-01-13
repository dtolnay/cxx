fn main() {
    cxx_build::bridge("src/main.rs")
        .file("src/main.cc")
        .compile("demo-dyntrait");
}

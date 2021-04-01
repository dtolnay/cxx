fn main() {
    cxx_build::bridges(&["src/lib.rs", "src/main.rs"])
        .compile("smessmer");
}

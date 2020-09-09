fn main() {
    cxx_build::bridge("src/main.rs").file("src/lib.cc").compile("no-causes");
}

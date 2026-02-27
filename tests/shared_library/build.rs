// build.rs

fn main() {
    // Read the target being used for the current build
    if let Ok(target) = std::env::var("TARGET") {
        // make env!("TARGET_TRIPLE") available
        println!("cargo:rustc-env=TARGET_TRIPLE={}", target);
    }
}

pub fn is_cmake_cargo_build() -> bool {
    std::env::var("CMAKECARGO_BUILD_DIR").is_ok()
}

pub fn build_script() {
    if !is_cmake_cargo_build() {
        panic!("CMAKECARGO_BUILD_DIR environment variable not set - build must be initiated from CMake.")
    }
    
    let search_dirs: Vec<_> = std::env::var("CMAKECARGO_LINK_DIRECTORIES")
        .iter()
        .flat_map(|v| v.split(";").map(std::string::ToString::to_string))
        .collect();    

    let libraries: Vec<_> = std::env::var("CMAKECARGO_LINK_LIBRARIES")
        .iter()
        .flat_map(|v| v.split(";").map(std::string::ToString::to_string))
        .collect();

    for dir in &search_dirs {
        println!("cargo:rustc-link-search={}", dir);
    }

    for library in &libraries {
        println!("cargo:rustc-link-lib={}", library);
    }
}

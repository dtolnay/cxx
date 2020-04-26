cmake_minimum_required(VERSION 3.10)
project(rust-lib-cpp-exe LANGUAGES CXX)

add_crate(rust/Cargo.toml)

add_executable(cpp-exe main.cpp)
target_link_libraries(cpp-exe PUBLIC rust-lib)
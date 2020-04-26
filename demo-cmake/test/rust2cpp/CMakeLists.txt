cmake_minimum_required(VERSION 3.10)
project(rust-lib-cpp-exe LANGUAGES CXX)

add_subdirectory(demo-rust-lib)

add_executable(cpp-exe main.cpp)
target_link_libraries(cpp-exe PUBLIC demo-rust-lib)
target_include_directories(cpp-exe PRIVATE ${CMAKE_CURRENT_SOURCE_DIR}/rust/demo-cxx)

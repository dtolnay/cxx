if (POLICY CMP0048)
  cmake_policy(SET CMP0048 NEW)
endif ()

project(cxx_example)
cmake_minimum_required(VERSION 3.15)


set (CMAKE_CXX_STANDARD 17)

add_subdirectory(rust_part)

add_executable(main main.cpp)

include_directories(${CMAKE_BINARY_DIR}/rust_part)
include_directories(include)

add_dependencies(main rust_part)

get_target_property(RUST_PART_DIR rust_part LOCATION)
target_link_libraries(main ${RUST_PART_DIR}/librust_part.so)

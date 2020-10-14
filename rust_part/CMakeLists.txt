if (CMAKE_BUILD_TYPE STREQUAL "Debug")
    set(CARGO_CMD cargo build)
    set(TARGET_DIR "debug")
else ()
    set(CARGO_CMD cargo build --release)
    set(TARGET_DIR "release")
endif ()

set(RUST_PART_SO "${CMAKE_CURRENT_BINARY_DIR}/${TARGET_DIR}/librust_part.so")

add_custom_target(rust_part ALL
    COMMENT "Compiling client module"
    COMMAND CARGO_TARGET_DIR=${CMAKE_CURRENT_BINARY_DIR} ${CARGO_CMD} 
    COMMAND cp ${RUST_PART_SO} ${CMAKE_CURRENT_BINARY_DIR}
    COMMAND cxxbridge src/lib.rs > ${CMAKE_CURRENT_BINARY_DIR}/rust_part.hpp
    WORKING_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR})
set_target_properties(rust_part PROPERTIES LOCATION ${CMAKE_CURRENT_BINARY_DIR})

add_test(NAME rust_part_test 
    COMMAND cargo test
    WORKING_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR})


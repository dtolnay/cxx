cmake_minimum_required(VERSION 2.8.12)

# Project configuration.
project(mmscenegraph)
set(PROJECT_VERSION_MAJOR 0)
set(PROJECT_VERSION_MINOR 1)
set(PROJECT_VERSION_PATCH 0)
set(PROJECT_VERSION "${PROJECT_VERSION_MAJOR}.${PROJECT_VERSION_MINOR}.${PROJECT_VERSION_PATCH}")
set(PROJECT_HOMEPAGE_URL "https://github.com/david-cattermole/cxx-demo-example")
set(PROJECT_DESCRIPTION "Demo project for the cxx rust crate.")
set(PROJECT_AUTHOR "David Cattermole")
set(PROJECT_COPYRIGHT "2020, David Cattermole.")


# Static build information.
set(target_release_lib_name "libmmscenegraph")
set(target_test_exe_name "mmscenegraph_tests")
set(rust_lib_name "mmscenegraph")

# Expected inputs.
set(RUST_BUILD_DIR "/path/to/rust/build/directory/" CACHE PATH
  "The path to the compiled rust directory.")
set(RUST_INCLUDE_DIR "/path/to/rust/include/directory/" CACHE PATH
  "The path to the generated include.")

# C++ Source Code
set(lib_source_files
  src/_cxxbridge.cpp
  src/cxx.cc
  src/lib.cpp
  )

set(test_source_files
  tests/main.cpp
  tests/test_a.cpp
  tests/test_b.cpp
  )

# Enable c++11
if (CMAKE_VERSION VERSION_LESS "3.1")
  if (CMAKE_CXX_COMPILER_ID STREQUAL "GNU")
    set (CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} -std=c++11")
  endif ()
else ()
  set (CMAKE_CXX_STANDARD 11)
endif ()

# Find the Rust compiled library
if (MSVC)
  set(rust_staticlib_name "${rust_lib_name}.lib")

  # Get the Rust Library .lib (for Windows).
  message("Finding: ${rust_staticlib_name}")
  message("RUST BUILD DIR: ${RUST_BUILD_DIR}")
  find_path(RUST_LINKTIME_LIBRARY_FILE ${rust_staticlib_name}
    HINTS ${RUST_BUILD_DIR}
    PATHS ${RUST_BUILD_DIR}
    )
  if(EXISTS ${RUST_LINKTIME_LIBRARY_FILE})
    set(RUST_LINKTIME_LIBRARY_FILE ${RUST_LINKTIME_LIBRARY_FILE}/${rust_staticlib_name})
  endif()

elseif (UNIX)
  set(rust_archive_name "lib${rust_lib_name}.a")

  # Get the Rust Library .a (for Linux).
  message("Finding: ${rust_archive_name}")
  message("RUST BUILD DIR: ${RUST_BUILD_DIR}")
  find_path(RUST_LINKTIME_LIBRARY_FILE ${rust_archive_name}
    HINTS ${RUST_BUILD_DIR}
    PATHS ${RUST_BUILD_DIR}
    )
  if(EXISTS ${RUST_LINKTIME_LIBRARY_FILE})
    set(RUST_LINKTIME_LIBRARY_FILE ${RUST_LINKTIME_LIBRARY_FILE}/${rust_archive_name})
  endif()
  
else ()
  message(FATAL_ERROR "Only Linux and Windows are supported.")
endif ()

message("Rust Library: " ${RUST_LINKTIME_LIBRARY_FILE})


# Add 'mmscenegraph' library as a C++ wrapper around the Rust library.
add_library(${target_release_lib_name} SHARED ${lib_source_files})
target_link_libraries(${target_release_lib_name}
  ${RUST_LINKTIME_LIBRARY_FILE})
target_include_directories(${target_release_lib_name}
  PUBLIC include
  PRIVATE src
  )
target_include_directories(${target_release_lib_name}
  PUBLIC ${RUST_INCLUDE_DIR}
  )
if (MSVC)
  target_link_libraries(${target_release_lib_name} ws2_32 userenv advapi32 shell32 msvcrt)
elseif (UNIX)
  target_link_libraries(${target_release_lib_name} pthread m dl)
endif ()


# Add C++ 'mmscenegraph' test executable
add_executable(${target_test_exe_name} ${test_source_files}
  ${lib_source_files})
target_link_libraries(${target_test_exe_name}
  ${target_release_lib_name})
target_include_directories(${target_test_exe_name}
  PUBLIC include
  PRIVATE src
  )
target_include_directories(${target_test_exe_name}
  PUBLIC ${RUST_INCLUDE_DIR}
  )

# Include public header files for installation
set(header_files
  # C++ public header files.
  include/mmscenegraph.h

  # Include internal generated headers, using 'cbindgen' and 'cxx'.
  include/cxx.h
  include/mmscenegraph/_mmscenegraph.h
  include/mmscenegraph/_cbindgen.h
  include/mmscenegraph/_cxxbridge.h
  )

# Install
install(FILES
  DESTINATION "bin")
install(DIRECTORY
  include/
  DESTINATION "include"
  FILES_MATCHING PATTERN "*.h")
install(
  TARGETS
  ${target_release_lib_name}
  ${target_test_exe_name}
  ARCHIVE DESTINATION "lib"
  LIBRARY DESTINATION "lib"
  RUNTIME DESTINATION "bin")

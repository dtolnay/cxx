cmake_minimum_required(VERSION 3.10)

# search for Cargo here and set up a bunch of cool flags and stuff
include(FindPackageHandleStandardArgs)

find_program(CARGO_EXECUTABLE cargo HINTS $ENV{HOME}/.cargo/bin)
find_program(RUSTC_EXECUTABLE rustc HINTS $ENV{HOME}/.cargo/bin)

set(CARGO_BUILD_FLAGS "" CACHE STRING "Flags to pass to cargo build")
set(CARGO_BUILD_FLAGS_DEBUG "" CACHE STRING
    "Flags to pass to cargo build in Debug configuration")
set(CARGO_BUILD_FLAGS_RELEASE --release CACHE STRING
    "Flags to pass to cargo build in Release configuration")
set(CARGO_BUILD_FLAGS_MINSIZEREL --release CACHE STRING
    "Flags to pass to cargo build in MinSizeRel configuration")
set(CARGO_BUILD_FLAGS_RELWITHDEBINFO --release CACHE STRING
    "Flags to pass to cargo build in RelWithDebInfo configuration")

set(CARGO_RUST_FLAGS "" CACHE STRING "Flags to pass to rustc")
set(CARGO_RUST_FLAGS_DEBUG "" CACHE STRING
    "Flags to pass to rustc in Debug Configuration")
set(CARGO_RUST_FLAGS_RELEASE "" CACHE STRING
    "Flags to pass to rustc in Release Configuration")
set(CARGO_RUST_FLAGS_MINSIZEREL -C opt-level=z CACHE STRING
    "Flags to pass to rustc in MinSizeRel Configuration")
set(CARGO_RUST_FLAGS_RELWITHDEBINFO -g CACHE STRING
    "Flags to pass to rustc in RelWithDebInfo Configuration")
    
if (WIN32)
    set(CARGO_BUILD_SCRIPT cargo_build.cmd)
    set(CARGO_BUILD ${CARGO_BUILD_SCRIPT})
else()
    set(CARGO_BUILD_SCRIPT cargo_build.sh)
    set(CARGO_BUILD ./${CARGO_BUILD_SCRIPT})
endif()


execute_process(
    COMMAND ${CARGO_EXECUTABLE} --version --verbose
    OUTPUT_VARIABLE CARGO_VERSION_RAW)

if (CARGO_VERSION_RAW MATCHES "cargo ([0-9]+)\\.([0-9]+)\\.([0-9]+)")
    set(CARGO_VERSION_MAJOR "${CMAKE_MATCH_1}")
    set(CARGO_VERSION_MINOR "${CMAKE_MATCH_2}")
    set(CARGO_VERSION_PATCH "${CMAKE_MATCH_3}")
    set(CARGO_VERSION "${CARGO_VERSION_MAJOR}.${CARGO_VERSION_MINOR}.${CARGO_VERSION_PATCH}")
else()
    message(
        FATAL_ERROR
        "Failed to parse cargo version. `cargo --version` evaluated to (${CARGO_VERSION_RAW})")
endif()

execute_process(
    COMMAND ${RUSTC_EXECUTABLE} --version --verbose
    OUTPUT_VARIABLE RUSTC_VERSION_RAW)

if (NOT CARGO_TARGET)
    if (WIN32)
        if (CMAKE_VS_PLATFORM_NAME)
            if ("${CMAKE_VS_PLATFORM_NAME}" STREQUAL "Win32")
                set(CARGO_ARCH i686 CACHE STRING "Build for 32-bit x86")
            elseif("${CMAKE_VS_PLATFORM_NAME}" STREQUAL "x64")
                set(CARGO_ARCH x86_64 CACHE STRING "Build for 64-bit x86")
            elseif("${CMAKE_VS_PLATFORM_NAME}" STREQUAL "ARM64")
                set(CARGO_ARCH aarch64 CACHE STRING "Build for 64-bit ARM")
            else()
                message(WARNING "VS Platform '${CMAKE_VS_PLATFORM_NAME}' not recognized")
            endif()
        else ()
            if (CMAKE_SIZEOF_VOID_P EQUAL 8)
                set(CARGO_ARCH x86_64 CACHE STRING "Build for 64-bit x86")
            else()
                set(CARGO_ARCH i686 CACHE STRING "Build for 32-bit x86")
            endif()
        endif()

        set(CARGO_VENDOR "pc-windows" CACHE STRING "Build for Microsoft Windows")

        if ("${CMAKE_CXX_COMPILER_ID}" STREQUAL "GNU")
            set(CARGO_ABI gnu CACHE STRING "Build for linking with GNU")
        else()
            set(CARGO_ABI msvc CACHE STRING "Build for linking with MSVC")
        endif()

        set(CARGO_TARGET "${CARGO_ARCH}-${CARGO_VENDOR}-${CARGO_ABI}"
            CACHE STRING "Windows Target")
    elseif (RUSTC_VERSION_RAW MATCHES "host: ([a-zA-Z0-9_\\-]*)\n")
        set(CARGO_TARGET "${CMAKE_MATCH_1}" CACHE STRING "Default Host Target")
    else()
        message(
            FATAL_ERROR
            "Failed to parse rustc host target. `rustc --version --verbose` evaluated to:\n${RUSTC_VERSION_RAW}"
        )
    endif()
endif()

message(STATUS "Rust Target: ${CARGO_TARGET}")

find_package_handle_standard_args(
    Cargo
    REQUIRED_VARS CARGO_EXECUTABLE
    VERSION_VAR CARGO_VERSION)

function(_gen_config config_type use_config_dir)
    string(TOUPPER "${config_type}" UPPER_CONFIG_TYPE)

    if(use_config_dir)
        set(_DESTINATION_DIR ${CMAKE_BINARY_DIR}/${CMAKE_VS_PLATFORM_NAME}/${config_type})
    else()
        set(_DESTINATION_DIR ${CMAKE_BINARY_DIR})
    endif()

    set(CARGO_CONFIG ${_DESTINATION_DIR}/.cargo/config)

    file(WRITE ${CARGO_CONFIG}
"\
[build]
target-dir=\"cargo/build\"
")

    string(REPLACE ";" "\", \"" RUSTFLAGS "${CARGO_RUST_FLAGS}" "${CARGO_RUST_FLAGS_${UPPER_CONFIG_TYPE}}")

    if (RUSTFLAGS)
        file(APPEND ${CARGO_CONFIG}
            "rustflags = [\"${RUSTFLAGS}\"]\n")
    endif()

    if (CARGO_TARGET)
        set(CARGO_BUILD_FLAGS ${CARGO_BUILD_FLAGS} --target ${CARGO_TARGET})
    endif()

    string(REPLACE ";" " " _CARGO_BUILD_FLAGS
        "${CARGO_BUILD_FLAGS} ${CARGO_BUILD_FLAGS_${UPPER_CONFIG_TYPE}}")

    get_filename_component(_moddir ${CMAKE_CURRENT_LIST_FILE} DIRECTORY)

    configure_file(
        ${_moddir}/cmds/${CARGO_BUILD_SCRIPT}.in
        ${CMAKE_BINARY_DIR}${CMAKE_FILES_DIRECTORY}/${CARGO_BUILD_SCRIPT})

    file(COPY ${CMAKE_BINARY_DIR}${CMAKE_FILES_DIRECTORY}/${CARGO_BUILD_SCRIPT}
        DESTINATION ${_DESTINATION_DIR}
        FILE_PERMISSIONS OWNER_READ OWNER_WRITE OWNER_EXECUTE GROUP_READ
        GROUP_EXECUTE WORLD_READ WORLD_EXECUTE)
endfunction(_gen_config)

if (CMAKE_CONFIGURATION_TYPES)
    foreach(config_type ${CMAKE_CONFIGURATION_TYPES})
        _gen_config(${config_type} ON)
    endforeach()
elseif(CMAKE_BUILD_TYPE)
    _gen_config(${CMAKE_BUILD_TYPE} OFF)
else()
    message(STATUS "Defaulting Cargo to build debug")
    _gen_config(Debug OFF)
endif()

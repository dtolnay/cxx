# Shared Library Test

This test validates cxx-gen's ability to generate code for shared library/DLL scenarios with bidirectional function calls.

## What This Tests

This test specifically validates the cxx_gen functionality for generating:

1. **export_symbols** - List of mangled symbols the shared library exports (Rust functions callable from C++)
2. **import_symbols** - List of mangled symbols the EXE must export (C++ functions callable from Rust)
3. **import_thunks** - C++ code compiled into the DLL to dynamically load EXE's exports via GetProcAddress (Windows only)

### Platform-Specific Behavior

**Windows:**
- Uses `.def` files to control symbol exports/imports
- Generates thunks that use `GetProcAddress` to dynamically resolve symbols from the executable
- Thunks are compiled into the DLL

**Linux:**
- Uses a version script to control symbol visibility in the shared library
- Uses a dynamic list to export only the required symbols from the executable (more precise than `--export-dynamic`)
- The shared library resolves imported symbols directly (no thunks needed)
- Uses `rpath` with `$ORIGIN` for library loading

**macOS:**
- Uses `-undefined dynamic_lookup` to allow undefined symbols
- The executable makes symbols available to the shared library at runtime

## Structure

- `library/lib.rs` - Defines the cxx bridge with exported Rust functions and imported C++ functions
- `library/build.rs` - Uses `cxx_gen::generate_header_and_cc()` to generate .def files and thunks
- `tests/main.cc` - Test executable that implements C++ functions and calls library functions
- `tests/exe_functions.{h,cc}` - Implementation of functions exported by the executable
- `tests/test_shared_library.rs` - Integration test that verifies the generated files and builds/runs test exe

## How It Works

1. **Library Build** (`library/build.rs`):
   - Parses `lib.rs` using cxx_gen to extract bridge declarations
   - Generates `library.def` with export_symbols (functions DLL exports) on Windows
   - Generates `exe.def` with import_symbols (functions EXE must export) on Windows
   - Generates `thunks.cc` with import_thunks (GetProcAddress-based loaders on Windows)
   - On Linux, generates a version script to control symbol visibility
   - On macOS, generates a response file with `-U` flags for each import symbol
   - Compiles everything into `test_library.dll` (Windows) or `libtest_library.so` (Linux/macOS)

2. **Test Execution** (`tests/test_shared_library.rs`):
   - Detects the test's target platform and builds the library with the same target
   - Builds the library with `cargo build --manifest-path library/Cargo.toml --target <target>`
   - Verifies .def files contain correct mangled symbols (Windows only)
   - Compiles a test executable linking main.cc + generated lib.cc + shared library
   - **Windows**: Uses .def files, DLL import library, and MSVC linker
   - **Linux**: Uses `--export-dynamic` to export symbols from executable, and `rpath` for library loading
   - **macOS**: Uses `@loader_path` rpath and dynamic symbol lookup
   - Runs the executable to validate bidirectional calling works

3. **Test Executable** (`tests/main.cc`):
   - Implements `exe_callback()` and `exe_get_constant()` in `tests/exe_functions.cc`
   - Calls library's `get_magic_number()`, `multiply_values()`, `library_entry_point()`
   - All calls go through cxx bridge (not direct mangled names)

## Testing

The tests verify that:
- The library builds successfully on Windows, Linux, and macOS
- Exported functions can be called and return correct values
- Imported functions are called correctly by the library
- The entry point demonstrates the full round-trip
- Platform-specific symbol resolution mechanisms work correctly

Run with:
```bash
# Test for your current platform
cargo test

# Test for a specific target
cargo test --target x86_64-unknown-linux-gnu
cargo test --target x86_64-pc-windows-msvc
cargo test --target x86_64-apple-darwin
```

## Expected Behavior

- `get_magic_number()` calls `exe_get_constant()` (returns 1000) and adds 42, returning 1042
- `multiply_values(3, 4)` computes 3*4=12, then calls `exe_callback(12)` which doubles it to 24
- `library_entry_point()` calls both `exe_callback(100)` and `exe_get_constant()`, returning 200 + 1000 = 1200

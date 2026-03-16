use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

#[test]
fn test_shared_library_with_exe() {
    // this test verifies the cxx_gen functionality for shared libraries:
    // 1. export_symbols - mangled symbols the DLL exports
    // 2. import_symbols - mangled symbols the EXE must export for the DLL
    // 3. thunks - code compiled into DLL to dynamically load EXE's exports

    // build the library first (cargo won't build cdylib automatically for tests)
    // match the build profile (debug vs release) to the test's profile
    let profile = if cfg!(debug_assertions) { "debug" } else { "release" };
    let mut build_cmd = Command::new("cargo");
    build_cmd.args(&["build", "--manifest-path", "library/Cargo.toml"]);
    if !cfg!(debug_assertions) {
        build_cmd.arg("--release");
    }
    // Build for the same target as this test binary
    let target = env!("TARGET_TRIPLE");
    build_cmd.arg("--target").arg(target);
    let status = build_cmd.status().expect("failed to execute cargo build");
    assert!(status.success(), "failed to build test-library");

    // the library's build script copies artifacts to library/target/cxxbridge/<profile>/
    let artifacts_dir = PathBuf::from(format!("library/target/cxxbridge/{}", profile));

    // verify the exe symbols file was created (platform-specific format)
    let exe_symbols_filename = if cfg!(target_os = "windows") {
        "exe.def"
    } else if cfg!(target_os = "macos") {
        "exe_undefined.txt"
    } else {
        "exe.dynamic"
    };
    let exe_symbols_path = artifacts_dir.join(exe_symbols_filename);

    let exe_symbols = fs::read_to_string(&exe_symbols_path)
        .unwrap_or_else(|e| panic!("failed to read {}: {}", exe_symbols_filename, e));

    println!("{}:\n{}", exe_symbols_filename, exe_symbols);

    // verify exe symbols file contains the mangled import symbols
    assert!(exe_symbols.contains("exe_callback"), "{} should list exe_callback", exe_symbols_filename);
    assert!(exe_symbols.contains("exe_get_constant"), "{} should list exe_get_constant", exe_symbols_filename);

    // On Windows, also verify library.def contains the mangled export symbols
    if cfg!(target_os = "windows") {
        let dll_def_path = artifacts_dir.join("library.def");
        let dll_def = fs::read_to_string(&dll_def_path).expect("failed to read library.def");

        println!("library.def:\n{}", dll_def);

        assert!(dll_def.contains("EXPORTS"), "library.def should have EXPORTS section");
        assert!(dll_def.contains("get_magic_number"), "library.def should list get_magic_number");
        assert!(dll_def.contains("multiply_values"), "library.def should list multiply_values");
    }

    // get the library's target directory for linking
    let lib_metadata = cargo_metadata::MetadataCommand::new()
        .manifest_path("library/Cargo.toml")
        .exec()
        .unwrap();
    let lib_target_dir = lib_metadata.target_directory.as_std_path();

    // Determine the actual library output directory
    // When using --target explicitly, cargo puts artifacts in target/<target>/<profile>/
    let lib_output_dir = lib_target_dir.join(target).join(profile);

    // build a test executable that uses the exe symbols file to export symbols
    build_and_run_test_exe(&artifacts_dir, &lib_output_dir, &exe_symbols_path);
}

fn build_and_run_test_exe(artifacts_dir: &PathBuf, target_dir: &std::path::Path, exe_symbols_path: &PathBuf) {
    let test_dir = artifacts_dir.join("test_exe");
    fs::create_dir_all(&test_dir).expect("failed to create test dir");

    let target = env!("TARGET_TRIPLE");

    // compile to object file
    let mut cc_build = cc::Build::new();

    // set required env vars that cc crate needs
    if env::var("OUT_DIR").is_err() {
        env::set_var("OUT_DIR", &test_dir);
    }
    if env::var("OPT_LEVEL").is_err() {
        env::set_var("OPT_LEVEL", "0");
    }
    if env::var("TARGET").is_err() {
        env::set_var("TARGET", target);
    }
    if env::var("HOST").is_err() {
        env::set_var("HOST", target);
    }

    cc_build.cpp(true)
        .file("tests/main.cc")
        .file("tests/exe_functions.cc")
        .file(artifacts_dir.join("lib.cc"))  // include bridge implementation with exe wrappers
        .include(&artifacts_dir)
        .include("tests");

    if cfg!(target_os = "windows") {
        cc_build.flag("/EHsc");
    } else {
        cc_build.flag("-std=c++17");
    }

    let objects = cc_build.compile_intermediates();
    let exe_path = test_dir.join(if cfg!(target_os = "windows") { "test_exe.exe" } else { "test_exe" });

    // link with exe symbols file and DLL import library
    if cfg!(target_os = "windows") {
        let tool = cc::windows_registry::find_tool(target, "link.exe")
            .expect("failed to find MSVC link.exe");

        let mut cmd = tool.to_command();
        cmd.arg("/OUT:".to_string() + exe_path.to_str().unwrap());
        cmd.arg("/SUBSYSTEM:CONSOLE");
        cmd.arg("/DEF:".to_string() + exe_symbols_path.to_str().unwrap());
        cmd.arg("/LIBPATH:".to_string() + target_dir.to_str().unwrap());
        for obj in &objects {
            cmd.arg(obj);
        }
        cmd.arg(target_dir.join("test_library.dll.lib"));

        println!("linking exe: {:?}", cmd);
        let link_output = cmd.output().expect("failed to link test executable");

        println!("link stdout: {}", String::from_utf8_lossy(&link_output.stdout));
        if !link_output.stderr.is_empty() {
            println!("link stderr: {}", String::from_utf8_lossy(&link_output.stderr));
        }

        assert!(link_output.status.success(), "failed to link test executable");

        // copy DLL to exe directory
        let dll_path = target_dir.join("test_library.dll");
        let exe_dir_dll = test_dir.join("test_library.dll");
        fs::copy(&dll_path, &exe_dir_dll).expect("failed to copy DLL");
    } else {
        // on Unix (Linux/macOS), link the executable with the shared library
        let lib_name = if cfg!(target_os = "macos") {
            "libtest_library.dylib"
        } else {
            "libtest_library.so"
        };

        let mut cmd = Command::new("c++");
        cmd.arg("-o").arg(&exe_path);
        for obj in &objects {
            cmd.arg(obj);
        }

        // Link with the shared library
        cmd.arg(target_dir.join(lib_name));

        // Set rpath to allow the library to resolve symbols from the exe
        if cfg!(target_os = "macos") {
            cmd.arg("-Wl,-rpath,@loader_path");
            cmd.arg(format!("-Wl,-rpath,{}", target_dir.display()));
        } else {
            // Linux: use $ORIGIN and absolute path for rpath
            cmd.arg("-Wl,-rpath=$ORIGIN");
            cmd.arg(format!("-Wl,-rpath,{}", target_dir.display()));

            // Use dynamic list to export only the required symbols (better than --export-dynamic)
            cmd.arg(format!("-Wl,--dynamic-list={}", exe_symbols_path.display()));
        }

        println!("linking exe: {:?}", cmd);
        let link_output = cmd.output().expect("failed to link test executable");

        if !link_output.stdout.is_empty() {
            println!("link stdout: {}", String::from_utf8_lossy(&link_output.stdout));
        }
        if !link_output.stderr.is_empty() {
            println!("link stderr: {}", String::from_utf8_lossy(&link_output.stderr));
        }

        assert!(link_output.status.success(), "failed to link test executable");

        // Copy the shared library to the test directory for $ORIGIN rpath
        let lib_src = target_dir.join(lib_name);
        let lib_dest = test_dir.join(lib_name);
        fs::copy(&lib_src, &lib_dest).expect("failed to copy shared library");
    }

    println!("test executable created at: {}", exe_path.display());

    // run the test executable
    println!("\nRunning test executable...\n");
    // Convert to absolute path for Command to work properly on all platforms
    let exe_abs_path = std::fs::canonicalize(&exe_path)
        .unwrap_or_else(|e| panic!("failed to canonicalize exe path {:?}: {}", exe_path, e));
    let output = Command::new(&exe_abs_path)
        .current_dir(&test_dir)
        .output()
        .unwrap_or_else(|e| panic!("failed to run test executable at {:?}: {}", exe_abs_path, e));

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    print!("{}", stdout);
    if !stderr.is_empty() {
        eprint!("{}", stderr);
    }

    assert!(output.status.success(), "test executable failed with output:\n{}\n{}", stdout, stderr);
}

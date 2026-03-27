use std::env;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use rustc_version::{version, Version};

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let profile = env::var("PROFILE").unwrap();
    let test_artifacts_dir = manifest_dir.join(format!("target/cxxbridge/{}", profile));
    fs::create_dir_all(&test_artifacts_dir).unwrap();

    // generate the bridge using cxx_gen to get symbols and thunks
    let lib_source = fs::read_to_string(manifest_dir.join("lib.rs")).unwrap();
    let lib_tokens: proc_macro2::TokenStream = lib_source.parse().unwrap();
    let generated = cxx_gen::generate_header_and_cc(lib_tokens, &cxx_gen::Opt::default()).unwrap();

    // write header and implementation
    fs::write(out_dir.join("lib.h"), &generated.header).unwrap();
    fs::write(out_dir.join("lib.cc"), &generated.implementation).unwrap();

    // create EXE symbols file in OS-appropriate format
    // Use TARGET env var to get the target OS, not the host OS
    let target = env::var("TARGET").unwrap();
    let target_os = TargetOs::from(target.as_str());

    let exe_symbols_content = cxx_gen::format_import_symbols_for_linker(
        &generated.import_symbols(),
        target_os.as_str(),
    );

    let exe_symbols_path = match target_os {
        TargetOs::Windows => out_dir.join("exe.def"),
        TargetOs::Macos => out_dir.join("exe_undefined.txt"),
        TargetOs::Linux => out_dir.join("exe.dynamic"),
    };
    fs::write(&exe_symbols_path, exe_symbols_content).unwrap();

    // compile C++ code needed by the library
    // On Windows: compile thunks (which call back to the exe via GetProcAddress)
    // On Unix: no C++ code needed in the library (generate_import_thunks returns empty string)
    let thunks = generated.generate_import_thunks(target_os.as_str());
    if !thunks.is_empty() {
        // write thunks that will be compiled into the DLL
        let thunks_path = out_dir.join("thunks.cc");
        fs::write(&thunks_path, &thunks).unwrap();

        let mut build = cc::Build::new();
        build
            .cpp(true)
            .flag("/EHsc")
            .file(&thunks_path)
            .include(&out_dir)
            .include(manifest_dir.parent().unwrap().join("tests"));  // for exe_functions.h
        build.compile("cxx-test-shared-library");
    }

    // on Windows, use the .def file for exports
    if target_os == TargetOs::Windows {
        // create DLL .def file with export symbols (functions the DLL exports)
        let dll_def_content = cxx_gen::format_export_symbols_for_linker(
            &generated.export_symbols(),
            "windows",
        );
        let dll_def_path = out_dir.join("library.def");
        fs::write(&dll_def_path, dll_def_content).unwrap();

        println!("cargo:rustc-cdylib-link-arg=/DEF:{}", dll_def_path.display());

        // copy for the test to use
        fs::copy(&dll_def_path, test_artifacts_dir.join("library.def")).unwrap();
    } else if target_os == TargetOs::Macos {
        // Per ld(1) man page: "-U symbol_name: Specified that it is ok for symbol_name to
        // have no definition. With -two_levelnamespace, the resulting symbol will be marked
        // dynamic_lookup which means dyld will search all loaded images."
        //
        // The Rust code in the library calls the cxxbridge wrapper functions (import_symbols),
        // which are implemented in lib.cc that's compiled into the executable.
        println!("cargo:rustc-cdylib-link-arg=-Wl,@{}", exe_symbols_path.display());
    } else {
        // on Linux, create a version script to export symbols and allow undefined symbols
        let mut version_script = Vec::new();
        writeln!(version_script, "{{").unwrap();
        writeln!(version_script, "  global:").unwrap();
        for sym in &generated.export_symbols() {
            writeln!(version_script, "    {};", sym).unwrap();
        }
        writeln!(version_script, "  local: *;").unwrap();
        writeln!(version_script, "}};").unwrap();
        let version_script_path = out_dir.join("libtest_library.version");
        fs::write(&version_script_path, version_script).unwrap();

        let curr = version().unwrap();
        let is_broken_version_script = curr < Version::parse("1.90.0").unwrap();
        if !is_broken_version_script {
            println!("cargo:rustc-cdylib-link-arg=-Wl,--version-script={}", version_script_path.display());
        }
        println!("cargo:rustc-cdylib-link-arg=-Wl,--allow-shlib-undefined");
    }

    // expose paths for the test to use
    println!("cargo:rustc-env=EXE_SYMBOLS_PATH={}", exe_symbols_path.display());

    // copy generated files to a predictable location for the test to find
    fs::copy(&exe_symbols_path, test_artifacts_dir.join(exe_symbols_path.file_name().unwrap())).unwrap();
    fs::copy(out_dir.join("lib.h"), test_artifacts_dir.join("lib.h")).unwrap();
    fs::copy(out_dir.join("lib.cc"), test_artifacts_dir.join("lib.cc")).unwrap();

    println!("cargo:rerun-if-changed=lib.rs");
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TargetOs {
    Windows,
    Macos,
    Linux,
}

impl From<&str> for TargetOs {
    fn from(target: &str) -> Self {
        if target.contains("windows") {
            TargetOs::Windows
        } else if target.contains("darwin") || target.contains("ios") {
            TargetOs::Macos
        } else {
            TargetOs::Linux
        }
    }
}

impl TargetOs {
    fn as_str(self) -> &'static str {
        match self {
            TargetOs::Windows => "windows",
            TargetOs::Macos => "macos",
            TargetOs::Linux => "linux",
        }
    }
}

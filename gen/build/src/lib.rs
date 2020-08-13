//! The CXX code generator for constructing and compiling C++ code.
//!
//! This is intended to be used from Cargo build scripts to execute CXX's
//! C++ code generator, set up any additional compiler flags depending on
//! the use case, and make the C++ compiler invocation.
//!
//! <br>
//!
//! # Example
//!
//! Example of a canonical Cargo build script that builds a CXX bridge:
//!
//! ```no_run
//! // build.rs
//!
//! fn main() {
//!     cxx_build::bridge("src/main.rs")
//!         .file("../demo-cxx/demo.cc")
//!         .flag_if_supported("-std=c++11")
//!         .compile("cxxbridge-demo");
//!
//!     println!("cargo:rerun-if-changed=src/main.rs");
//!     println!("cargo:rerun-if-changed=../demo-cxx/demo.h");
//!     println!("cargo:rerun-if-changed=../demo-cxx/demo.cc");
//! }
//! ```
//!
//! A runnable working setup with this build script is shown in the
//! *demo-rs* and *demo-cxx* directories of [https://github.com/dtolnay/cxx].
//!
//! [https://github.com/dtolnay/cxx]: https://github.com/dtolnay/cxx
//!
//! <br>
//!
//! # Alternatives
//!
//! For use in non-Cargo builds like Bazel or Buck, CXX provides an
//! alternate way of invoking the C++ code generator as a standalone command
//! line tool. The tool is packaged as the `cxxbridge-cmd` crate.
//!
//! ```bash
//! $ cargo install cxxbridge-cmd  # or build it from the repo
//!
//! $ cxxbridge src/main.rs --header > path/to/mybridge.h
//! $ cxxbridge src/main.rs > path/to/mybridge.cc
//! ```

#![allow(
    clippy::inherent_to_string,
    clippy::needless_doctest_main,
    clippy::new_without_default,
    clippy::toplevel_ref_arg
)]

mod error;
mod gen;
mod paths;
mod syntax;

use crate::error::Result;
use crate::gen::Opt;
use anyhow::anyhow;
use std::fs;
use std::io::{self, Write};
use std::iter;
use std::path::Path;
use std::process;

/// This returns a [`cc::Build`] on which you should continue to set up any
/// additional source files or compiler flags, and lastly call its [`compile`]
/// method to execute the C++ build.
///
/// [`compile`]: https://docs.rs/cc/1.0.49/cc/struct.Build.html#method.compile
#[must_use]
pub fn bridge(rust_source_file: impl AsRef<Path>) -> cc::Build {
    bridges(iter::once(rust_source_file))
}

/// `cxx_build::bridge` but for when more than one file contains a
/// #\[cxx::bridge\] module.
///
/// ```no_run
/// let source_files = vec!["src/main.rs", "src/path/to/other.rs"];
/// cxx_build::bridges(source_files)
///     .file("../demo-cxx/demo.cc")
///     .flag_if_supported("-std=c++11")
///     .compile("cxxbridge-demo");
/// ```
#[must_use]
pub fn bridges(rust_source_files: impl IntoIterator<Item = impl AsRef<Path>>) -> cc::Build {
    // try to clean up everything so that we don't end up with dangling files if things get moved around
    if let Ok(out_dir) = paths::out_dir() {
        std::fs::remove_dir_all(&out_dir).ok();
        std::fs::create_dir(out_dir).ok();
    }
    let mut build = paths::cc_build();
    build.cpp(true);
    build.cpp_link_stdlib(None); // linked via link-cplusplus crate

    for path in rust_source_files {
        if let Err(err) = try_generate_bridge(&mut build, path.as_ref()) {
            let _ = writeln!(io::stderr(), "\n\ncxxbridge error: {:?}\n\n", anyhow!(err));
            process::exit(1);
        }
    }
    if let Err(err) = write_header_file() {
        writeln!(io::stderr(), "\n\ncxxbridge error: {:?}\n\n", anyhow!(err)).ok();
        process::exit(1);
    }
    build
}

fn try_generate_bridge(build: &mut cc::Build, rust_source_file: &Path) -> Result<()> {
    let header = gen::do_generate_header(rust_source_file, Opt::default());
    let manifest_header_path =
        paths::out_with_extension(rust_source_file, ".h", paths::RelativeToDir::Manifest)?;

    fs::create_dir_all(manifest_header_path.parent().unwrap())?;
    fs::write(&manifest_header_path, &header)?;

    let bridge = gen::do_generate_bridge(rust_source_file, Opt::default());
    let bridge_path =
        paths::out_with_extension(rust_source_file, ".cc", paths::RelativeToDir::Manifest)?;
    fs::write(&bridge_path, bridge)?;
    build.file(&bridge_path);

    // Write out headers relative to the workspace path as well so that includes relative to there work.
    let workspace_header_path =
        paths::out_with_extension(rust_source_file, ".h", paths::RelativeToDir::Workspace)?;

    fs::create_dir_all(workspace_header_path.parent().unwrap())?;
    fs::write(&workspace_header_path, &header)?;

    Ok(())
}

fn write_header_file() -> Result<()> {
    let ref cxx_h = paths::include_dir()?.join("rust").join("cxx.h");
    fs::create_dir_all(cxx_h.parent().unwrap()).ok();
    fs::remove_file(cxx_h).ok();
    fs::write(cxx_h, gen::include::HEADER)?;
    Ok(())
}

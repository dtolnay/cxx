use std::collections::BTreeMap;
use std::env;
use std::path::PathBuf;

pub fn include_dirs() -> impl Iterator<Item = PathBuf> {
    let mut env_include_dirs = BTreeMap::new();
    for (k, v) in env::vars_os() {
        let mut k = k.to_string_lossy().into_owned();
        // Only variables set from a build script of direct dependencies are
        // observable. That's exactly what we want! Your crate needs to declare
        // a direct dependency on the other crate in order to be able to
        // #include its headers.
        //
        // Also, they're only observable if the dependency's manifest contains a
        // `links` key. This is important because Cargo imposes no ordering on
        // the execution of build scripts without a `links` key. When exposing a
        // generated header for the current crate to #include, we need to be
        // sure the dependency's build script has already executed and emitted
        // that generated header.
        //
        // References:
        //   - https://doc.rust-lang.org/cargo/reference/build-scripts.html#the-links-manifest-key
        //   - https://doc.rust-lang.org/cargo/reference/build-script-examples.html#using-another-sys-crate
        if k.starts_with("DEP_") {
            if k.ends_with("_CXXBRIDGE_INCLUDE") {
                // Tweak to ensure sorted before the other one, for the same
                // reason as the comment on ordering of include_dir vs crate_dir
                // above.
                k.replace_range(k.len() - "INCLUDE".len().., "0");
                env_include_dirs.insert(k, PathBuf::from(v));
            } else if k.ends_with("_CXXBRIDGE_CRATE") {
                k.replace_range(k.len() - "CRATE".len().., "1");
                env_include_dirs.insert(k, PathBuf::from(v));
            }
        }
    }
    env_include_dirs.into_iter().map(|entry| entry.1)
}

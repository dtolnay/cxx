alias w := watch
alias b := build
alias t := test

watch +WATCH_TARGET='test':
    watchexec -rc -w BUILD.bazel -w tests -w src -w gen -w macro -w syntax -w kj-rs -- just {{WATCH_TARGET}}

build:
    bazel build //...

test:
    bazel test //...

clippy:
    bazel build --config=clippy  //...

expand:
    bazel build //kj-rs/tests:expand-rust_test

cargo-update:
    bazel run //third-party:vendor

format: rustfmt clang-format

rustfmt:
    bazel run @rules_rust//:rustfmt

clang-format:
    clang-format -i kj-rs/*.h kj-rs/*.c++ kj-rs/tests/*.h kj-rs/tests/*.c++


compile-commands:
    bazel run @hedron_compile_commands//:refresh_all
    
# called by rust-analyzer discoverConfig (quiet recipe with no output)
@_rust-analyzer:
  rm -rf ./rust-project.json
  # rust-analyzer doesn't like stderr output, redirect it to /dev/null
  bazel run @rules_rust//tools/rust_analyzer:discover_bazel_rust_project 2>/dev/null

alias w := watch
alias b := build
alias t := test

watch +WATCH_TARGET='test':
    watchexec -rc -w tests -w src -w gen -w macro -w syntax -- just {{WATCH_TARGET}}

build:
    bazel build //...

test:
    bazel test //...

clippy:
    bazel build --config=clippy  //...

cargo-update:
    bazel run //third-party:vendor
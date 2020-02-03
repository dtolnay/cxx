#!/usr/bin/env bash

cargo generate-lockfile && \
cargo vendor --versioned-dirs --locked && \
cargo raze && \
cat >> $(find vendor/cxx-* -name "BUILD.bazel") <<- EOM

cc_library(
 name="cxx_cc_library",
 hdrs=glob(["include/**/*.h"]),
 srcs=glob(["src/**/*.cc"]),
)

EOM

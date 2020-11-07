#!/usr/bin/env bash

# Store the current working directory, to return to.
CWD=`pwd`

# Path to this script.
THIS_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"

# The root of this project.
ROOT=`readlink -f ${THIS_DIR}`

# Install directory
INSTALL_DIR="${ROOT}/install"

# Where to find the Rust libraries and headers.
RUST_BUILD_DIR="${ROOT}/target/release/"
RUST_INCLUDE_DIR="${ROOT}/include/"

# Set up environment for compatiblity with VFX Reference Platform
export CC=/opt/rh/devtoolset-6/root/usr/bin/gcc
export CXX=/opt/rh/devtoolset-6/root/usr/bin/g++
export _GLIBCXX_USE_CXX11_ABI=0  # Use the old std::string and std::list ABI.

# Build Rust
#
# Assumes 'cxxbridge-cmd' and 'cbindgen' is installed.
cxxbridge --header --output "${ROOT}/include/cxx.h"
cxxbridge src/cxxbridge.rs \
          --header --output "${ROOT}/include/mmscenegraph/_cxxbridge.h"
cxxbridge src/cxxbridge.rs \
          --cxx-impl-annotations "__attribute__((visibility(\"default\")))" \
          --output "${ROOT}/src/_cxxbridge.cpp"
cbindgen --config cbindgen.toml \
         --crate mmscenegraph \
         --output "${ROOT}/include/mmscenegraph/_cbindgen.h"
cargo build --release

# Build C++
mkdir -p build
cd build
cmake \
    -DCMAKE_BUILD_TYPE=Release \
    -DCMAKE_INSTALL_PREFIX=${INSTALL_DIR} \
    -DRUST_BUILD_DIR=${RUST_BUILD_DIR} \
    -DRUST_INCLUDE_DIR=${RUST_INCLUDE_DIR} \
    ..
make clean
make all
make install

# Return back project root directory.
cd ${CWD}

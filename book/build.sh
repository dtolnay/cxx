#!/bin/bash

set -e

cd "$(dirname "$0")"

if [ -f ./mdbook ]; then
    ./mdbook build
else
    mdbook build
fi

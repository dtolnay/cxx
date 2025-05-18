#!/bin/bash

set -eu

args=("$@")
output_filename=
version_script=

for i in ${!args[@]}; do
    if [[ "${args[$i]}" == -o ]]; then
        output_filename=$(basename "${args[i+1]}")
    elif [[ "${args[$i]}" == -Wl,--version-script=* ]]; then
        version_script=$i
    fi
done

if [ -n "$version_script" -a -f "linker/${output_filename}.map" ]; then
    args[$version_script]="-Wl,--version-script=linker/${output_filename}.map"
fi

clang "${args[@]}"

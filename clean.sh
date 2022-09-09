#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'


if [[ -n $(grep foo-dynamic Cargo.toml) ]]; then
    # poor mans cargo rm
    cat Cargo.toml | grep -v foo-dynamic > Cargo.toml.copy
    mv Cargo.toml{.copy,}
fi

if [[ -d foo-dynamic ]]; then
    rm -rf foo-dynamic
fi

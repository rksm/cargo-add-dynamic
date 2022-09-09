#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'


if [[ -n $(grep foo-dynamic Cargo.toml) ]]; then
    cargo rm foo
fi

if [[ -d foo-dynamic ]]; then
    rm -rf foo-dynamic
fi

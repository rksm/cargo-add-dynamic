#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'

./clean.sh

RUST_BACKTRACE=1 RUST_LOG=trace cargo run -- foo

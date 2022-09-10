#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'

# A simple test that adds a dynamic dependency to the foo placeholder package
# (https://crates.io/crates/foo) and removes it again

./scripts/clean.sh

./scripts/install.sh

# RUST_BACKTRACE=1 RUST_LOG=trace cargo run -- foo
cargo add-dynamic foo -v

./scripts/clean.sh

#!/bin/bash
if [ ! -f Cargo.lock ]; then
    echo 1>&2 "Run this from a top-level workspace directory."
    exit 2
fi

set -eo pipefail
cargo fmt
cargo clippy --no-deps
cargo clippy --tests --no-deps
cargo doc --document-private-items --no-deps
cargo build --workspace --all-targets
export RUST_BACKTRACE=1
cargo test --workspace

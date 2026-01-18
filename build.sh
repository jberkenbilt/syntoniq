#!/bin/bash
if [ ! -f Cargo.lock ]; then
    echo 1>&2 "Run this from a top-level workspace directory."
    exit 2
fi

set -eo pipefail
cargo fmt
cargo clippy --no-deps
cargo clippy --tests --no-deps
# Avoid --document-private-items -- generates warnings with csound docs
cargo doc --no-deps
cargo build --workspace --all-targets "$@"
if [ -z "$GITHUB_RUN_ID" ]; then
    # TODO: build in CI once the zola issue is resolved
    ./manual/build
fi
export RUST_BACKTRACE=1
cargo test --workspace "$@"

#!/bin/bash
if [ ! -f Cargo.lock ]; then
    echo 1>&2 "Run this from a top-level workspace directory."
    exit 2
fi

# Note: any arguments are executed after automated tests and before
# generating coverage. You can run
#
# ./test_coverage $SHELL
#
# to get a shell prompt at that stage and run manual tests to have
# them count for coverage, or you could invoke a script that runs
# integration tests.

# Required:
# - cargo install grcov
# - rustup component add llvm-tools-preview

set -xeo pipefail
rm -rf .grcov
export RUSTFLAGS="-Cinstrument-coverage"
export LLVM_PROFILE_FILE="$PWD/.grcov/%p-%m.profraw"
# Running ./build.sh ensures that rust code exercised through external
# tests or doc tests are also included in coverage output.
$(dirname $0)/build.sh

# For coverage, exercise tracing. It requires an environment variable
# to set, which can't be set with safe code in rust. This allows us to
# keep 100% coverage for model.rs.
CLICOLOR_FORCE=1 \
    SYNTONIQ_TRACE_LEXER=1 \
    target/debug/tokenize common/parsing-tests/errors-3.stq >/dev/null 2>&1

if [ "$1" != "" ]; then
    "$@"
fi
# Add --branch for branch coverage, but it doesn't seem to give useful
# information (2024-09).
rm -rf target/debug/coverage
grcov . -s . --binary-path target/debug/ -t html --ignore-not-existing -o target/debug/coverage/

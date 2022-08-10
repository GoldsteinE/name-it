#!/bin/sh

set -e

cd "$(dirname "$0")"/adapt-crate-tests

# futures-rs
cargo r -- --prefix futures-channel- 'https://github.com/rust-lang/futures-rs.git' ../../tests futures-channel/tests

cd ../..
cargo fmt

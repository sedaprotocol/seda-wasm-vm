#!/bin/sh
set -e

rm -rf /.cargo/registry/index
cargo fetch

echo "Starting aarch64-unknown-linux-musl build"
export CC=/opt/aarch64-linux-musl-cross/bin/aarch64-linux-musl-gcc
cargo build --release --target aarch64-unknown-linux-musl
unset CC

echo "Starting x86_64-unknown-linux-musl build"
cargo build --release --target x86_64-unknown-linux-musl

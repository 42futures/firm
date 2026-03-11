#!/bin/bash
set -euo pipefail

# Build firm_ffi for iOS targets and generate Swift bindings

echo "Building for iOS device (aarch64-apple-ios)..."
cargo build -p firm_ffi --release --target aarch64-apple-ios

echo "Building for iOS simulator (aarch64-apple-ios-sim)..."
cargo build -p firm_ffi --release --target aarch64-apple-ios-sim

echo "Generating Swift bindings..."
cargo run -p firm_ffi --bin uniffi-bindgen generate \
    --library target/release/libfirm_ffi.dylib \
    --language swift \
    --out-dir ./firm_ffi/generated

echo "Done. Swift bindings are in firm_ffi/generated/"

#!/usr/bin/env bash
#
# Local CI for the simple-ui workspace (mirrors .github/workflows/rust.yml).
set -euo pipefail

echo "Running CI for simple-ui..."

echo "==> Checking formatting..."
cargo fmt --check

echo "==> Clippy (deny warnings)..."
cargo clippy --workspace --all-targets -- -D warnings

echo "==> Building workspace..."
cargo build

echo "==> Running tests..."
cargo test

echo "CI completed successfully!"

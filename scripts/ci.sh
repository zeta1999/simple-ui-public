#!/bin/bash

# Exit immediately if a command exits with a non-zero status
set -e

echo "Running CI Script..."

echo "==> Formatting Rust code..."
cargo fmt --all -- --check

echo "==> Running Clippy (Rust linter)..."
cargo clippy --all-targets --all-features -- -D warnings

echo "==> Running Rust tests..."
cargo test --all

echo "==> Building Graphical Target..."
cd graphical
npm install
npm run build
cd ..

echo "==> CI Completed Successfully!"

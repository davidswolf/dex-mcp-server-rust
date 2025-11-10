#!/bin/bash
# CI validation script - runs the same checks as GitHub Actions
set -e

echo "======================================"
echo "Running CI checks locally..."
echo "======================================"
echo

echo "1. Checking code formatting..."
cargo fmt --all -- --check
echo "✓ Code formatting check passed"
echo

echo "2. Running clippy..."
cargo clippy --all-targets --all-features -- -D warnings
echo "✓ Clippy passed"
echo

echo "3. Running tests..."
cargo test --verbose
echo "✓ Tests passed"
echo

echo "4. Building release..."
cargo build --release --verbose
echo "✓ Release build passed"
echo

echo "5. Checking documentation..."
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --all-features
echo "✓ Documentation check passed"
echo

echo "======================================"
echo "All CI checks passed! ✓"
echo "======================================"

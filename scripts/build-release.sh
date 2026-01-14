#!/bin/bash
set -euo pipefail

# Build release binary for ralph CLI

echo "Building ralph release binary..."

# Build in release mode
cargo build --release

# Get version from Cargo.toml
VERSION=$(cargo pkgid | sed 's/.*#//')

echo ""
echo "Build complete!"
echo "Version: $VERSION"
echo "Binary: target/release/ralph"
echo ""

# Show version from built binary
./target/release/ralph --version

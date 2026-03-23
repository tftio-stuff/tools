#!/usr/bin/env bash
set -euo pipefail

if [ $# -lt 1 ]; then
    echo "Usage: $0 <crate-name> [--lib]"
    echo "  Creates a new crate in the workspace"
    exit 1
fi

CRATE_NAME="$1"
IS_LIB="${2:-}"
CRATE_DIR="crates/$CRATE_NAME"
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"

if [ -d "$ROOT_DIR/$CRATE_DIR" ]; then
    echo "Error: $CRATE_DIR already exists"
    exit 1
fi

mkdir -p "$ROOT_DIR/$CRATE_DIR/src"

if [ "$IS_LIB" = "--lib" ]; then
    cat > "$ROOT_DIR/$CRATE_DIR/src/lib.rs" << 'RUST'
//! TODO: Add crate documentation.
RUST
else
    cat > "$ROOT_DIR/$CRATE_DIR/src/main.rs" << 'RUST'
//! TODO: Add crate documentation.

fn main() {
    println!("Hello from new crate");
}
RUST
fi

cat > "$ROOT_DIR/$CRATE_DIR/Cargo.toml" << TOML
[package]
name = "$CRATE_NAME"
version = "2.2.0"
edition.workspace = true
rust-version.workspace = true
license.workspace = true
repository.workspace = true
description = "TODO: Add description"

[lints]
workspace = true

[dependencies]
TOML

echo ""
echo "Created $CRATE_DIR"
echo ""
echo "Next steps:"
echo "  1. Add \"$CRATE_DIR\" to [workspace.members] in Cargo.toml"
echo "  2. Add entry to release-please-config.json"
echo "  3. Add entry to the linked-versions component list"
echo "  4. Add entry to .release-please-manifest.json"
echo "  5. Run: cargo check -p $CRATE_NAME"

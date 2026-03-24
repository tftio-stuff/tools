# tftio-stuff/tools - Cargo Workspace
#
# prek.toml is the source of truth for quality checks.
# Recipes below delegate to prek stages or provide direct shortcuts.

# Default recipe
default:
    @just --list

# --- prek-delegated workflows ---

# Auto-fix formatting and lint issues
dev:
    prek run --hook-stage manual

# Full CI pipeline (check-only, no file modifications)
ci:
    prek run --hook-stage pre-push --all-files

# --- Direct shortcuts (for one-off use) ---

# Build all crates in debug mode
build:
    cargo build --workspace

# Build all crates in release mode
build-release:
    cargo build --workspace --release

# Build a single crate
build-crate crate:
    cargo build -p {{ crate }}

# Run all tests
test:
    cargo test --workspace --verbose

# Run tests for a single crate
test-crate crate:
    cargo test -p {{ crate }} --verbose

# Format code (requires nightly)
format:
    cargo +nightly fmt --all

# Check formatting
format-check:
    cargo +nightly fmt --all -- --check

# Lint with clippy
lint:
    cargo clippy --workspace --all-targets -- -D warnings

# Workspace-wide base CLI UX consistency checks
cli-consistency:
    just cli-metadata-consistency
    cargo run -q -p tftio-unvenv -- --help >/dev/null
    cargo run -q -p tftio-gator -- --help >/dev/null
    cargo run -q -p tftio-todoer -- --help >/dev/null
    cargo run -q -p tftio-silent-critic -- --help >/dev/null
    cargo run -q -p tftio-prompter -- --help >/dev/null
    cargo run -q -p tftio-asana-cli -- --help >/dev/null
    cargo run -q -p tftio-bsky-comment-extractor -- --help >/dev/null
    out="$(cargo run -q -p tftio-gator -- claude --session abc --no-yolo --json 2>/dev/null || true)"; printf '%s' "$out" | grep '"ok"' >/dev/null; printf '%s' "$out" | grep '"command"' >/dev/null
    out="$(cargo run -q -p tftio-todoer -- list --all --json 2>/dev/null || true)"; printf '%s' "$out" | grep '"ok"' >/dev/null; printf '%s' "$out" | grep '"command"' >/dev/null
    out="$(cargo run -q -p tftio-silent-critic -- --json project init --name consistency-check 2>/dev/null || true)"; printf '%s' "$out" | grep '"ok"' >/dev/null; printf '%s' "$out" | grep '"command"' >/dev/null
    sh ./tests/cli/06-agent-mode.sh

# Shell-based smoke test for shared metadata-command UX
cli-metadata-consistency:
    ./scripts/test-cli-metadata-consistency.sh

# Security audit
audit:
    cargo audit

# Dependency compliance
deny:
    cargo deny check

# Clean build artifacts
clean:
    cargo clean

# Run a crate binary
run crate *args:
    cargo run -p {{ crate }} -- {{ args }}

# Scaffold a new crate
new-crate name:
    @./scripts/new-crate.sh {{ name }}

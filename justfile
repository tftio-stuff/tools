# tftio-stuff/tools - Cargo Workspace

# Default recipe
default:
    @just --list

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
    @if rustup toolchain list | grep -q nightly; then \
        cargo +nightly fmt --all; \
        echo "Code formatted"; \
    else \
        echo "Nightly toolchain required: rustup install nightly"; \
        exit 1; \
    fi

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

# Shell-based smoke test for shared metadata-command UX
cli-metadata-consistency:
    /Users/jfb/Projects/tools/feature-add-agent-help-to-all-tools/scripts/test-cli-metadata-consistency.sh

# Security audit
audit:
    cargo audit

# Dependency compliance
deny:
    cargo deny check

# Code quality checks
quality: format-check lint

# Full CI pipeline
ci: quality test build-release audit deny
    @echo "CI pipeline complete"

# Development workflow
dev: format lint test
    @echo "Development checks complete"

# Clean build artifacts
clean:
    cargo clean

# Run a crate binary
run crate *args:
    cargo run -p {{ crate }} -- {{ args }}

# Scaffold a new crate
new-crate name:
    @./scripts/new-crate.sh {{ name }}

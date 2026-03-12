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

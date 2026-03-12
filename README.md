# tftio tools

Cargo workspace monorepo containing Rust CLI tools.

| Crate | Description | Install |
|-------|-------------|---------|
| `tftio-prompter` | Compose reusable prompt snippets from markdown libraries | `cargo install tftio-prompter` |
| `tftio-unvenv` | Detect Python virtual environments not ignored by Git | `cargo install tftio-unvenv` |
| `tftio-asana-cli` | Interface to the Asana API | `cargo install tftio-asana-cli` |
| `tftio-todoer` | Global todo list manager for LLM agents | `cargo install tftio-todoer` |
| `tftio-cli-common` | Shared library (not installable) | `cargo add tftio-cli-common` |

## Install from cargo

```bash
cargo install tftio-prompter
cargo install tftio-unvenv
cargo install tftio-asana-cli
cargo install tftio-todoer
```

Or install all binary crates at once:

```bash
cargo install tftio-prompter tftio-unvenv tftio-asana-cli tftio-todoer
```

## Install from GitHub releases

Pre-built binaries for Linux (x86_64, aarch64) and macOS (Apple Silicon) are attached to each [GitHub release](https://github.com/tftio-stuff/tools/releases).

```bash
# Example: install prompter on macOS
curl -fsSL https://github.com/tftio-stuff/tools/releases/latest/download/prompter-aarch64-apple-darwin.tar.gz | tar xz
mv prompter /usr/local/bin/
```

## Build from source

```bash
git clone https://github.com/tftio-stuff/tools.git
cd tools
cargo build --workspace --release
```

Binaries are in `target/release/`.

## Development

Requires Rust 1.94.0+ and nightly for formatting.

```bash
just dev    # format + lint + test
just ci     # full pipeline
just test   # tests only
```

## License

MIT, except `tftio-todoer` which is CC0-1.0.

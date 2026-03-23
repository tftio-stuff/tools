#!/bin/sh
set -eu

repo_root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)

uv run python - <<'PY' "$repo_root"
import json
import pathlib
import sys
import tomllib

root = pathlib.Path(sys.argv[1])
crates = sorted((root / "crates").glob("*/Cargo.toml"))

versions = {}
bin_names = []
for cargo_toml in crates:
    data = tomllib.loads(cargo_toml.read_text())
    pkg = data.get("package", {})
    versions[str(cargo_toml.parent.relative_to(root))] = pkg.get("version")
    for bin_entry in data.get("bin", []):
        name = bin_entry.get("name")
        if name is not None:
            bin_names.append(name)
            if name.startswith("tftio-"):
                raise SystemExit(f"binary still has tftio- prefix: {name}")

unique_versions = sorted(set(versions.values()))
if len(unique_versions) != 1:
    raise SystemExit(f"workspace packages do not share one version: {versions}")

manifest = json.loads((root / ".release-please-manifest.json").read_text())
manifest_versions = sorted(set(manifest.values()))
if len(manifest_versions) != 1:
    raise SystemExit(f"release-please manifest versions are not unified: {manifest}")

config = json.loads((root / "release-please-config.json").read_text())
if config.get("separate-pull-requests") is not False:
    raise SystemExit("release-please still uses separate pull requests")

plugins = config.get("plugins", [])
plugin_types = []
linked = None
for plugin in plugins:
    if isinstance(plugin, str):
        plugin_types.append(plugin)
    elif isinstance(plugin, dict):
        plugin_types.append(plugin.get("type"))
        if plugin.get("type") == "linked-versions":
            linked = plugin

if "cargo-workspace" not in plugin_types:
    raise SystemExit("release-please config is missing the cargo-workspace plugin")
if linked is None:
    raise SystemExit("release-please config is missing the linked-versions plugin")

expected_components = sorted([
    "cli-common",
    "prompter",
    "unvenv",
    "asana-cli",
    "todoer",
    "silent-critic",
    "gator",
    "bsky-comment-extractor",
])
if sorted(linked.get("components", [])) != expected_components:
    raise SystemExit(f"linked-versions components mismatch: {linked}")

print("PASS: release config and binary naming are workspace-consistent")
PY

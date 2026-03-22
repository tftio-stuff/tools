#!/bin/sh
set -eu

script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
repo_root=$(CDPATH= cd -- "${script_dir}/../.." && pwd)
# shellcheck source=tests/cli/lib.sh
. "${script_dir}/lib.sh"

printf '== extracted boilerplate stays deleted ==\n'

assert_file_not_contains \
    "${repo_root}/crates/gator/src/main.rs" \
    "struct NoDoctor" \
    "gator should use the shared doctorless adapter"
assert_file_not_contains \
    "${repo_root}/crates/todoer/src/main.rs" \
    "struct NoDoctor" \
    "todoer should use the shared doctorless adapter"
assert_file_not_contains \
    "${repo_root}/crates/silent-critic/src/main.rs" \
    "struct NoDoctor" \
    "silent-critic should use the shared doctorless adapter"

assert_file_not_contains \
    "${repo_root}/crates/gator/src/main.rs" \
    "fn tool_spec()" \
    "gator should use shared workspace tool metadata"
assert_file_not_contains \
    "${repo_root}/crates/todoer/src/main.rs" \
    "fn tool_spec()" \
    "todoer should use shared workspace tool metadata"
assert_file_not_contains \
    "${repo_root}/crates/silent-critic/src/main.rs" \
    "fn tool_spec()" \
    "silent-critic should use shared workspace tool metadata"
assert_file_not_contains \
    "${repo_root}/crates/bsky-comment-extractor/src/main.rs" \
    "fn tool_spec()" \
    "bce should use shared workspace tool metadata"

assert_file_not_contains \
    "${repo_root}/crates/prompter/src/completions.rs" \
    "fn render_instructions(" \
    "prompter should rely on shared completion instruction rendering"
assert_file_not_contains \
    "${repo_root}/crates/prompter/src/doctor.rs" \
    "pub fn run_doctor_with_json" \
    "prompter should rely on shared doctor report rendering helpers"

assert_file_contains \
    "${repo_root}/crates/unvenv/src/main.rs" \
    "run_with_display_error_handler" \
    "unvenv should use the shared fatal runner helper"
assert_file_contains \
    "${repo_root}/crates/asana-cli/src/main.rs" \
    "run_with_display_error_handler" \
    "asana-cli should use the shared fatal runner helper"
assert_file_contains \
    "${repo_root}/crates/silent-critic/src/main.rs" \
    "render_response_parts" \
    "silent-critic should use shared response emitters for richer command outputs"
assert_file_not_contains \
    "${repo_root}/crates/silent-critic/src/main.rs" \
    "if json {" \
    "silent-critic should not keep local JSON/text branching boilerplate in main.rs"
assert_file_contains \
    "${repo_root}/crates/prompter/src/doctor.rs" \
    "DoctorReport::for_tool" \
    "prompter should build doctor reports through shared scaffolding"

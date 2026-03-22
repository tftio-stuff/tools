#!/bin/sh
set -eu

script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
# shellcheck source=tests/cli/lib.sh
. "${script_dir}/lib.sh"

printf '== bce primary invocation still parses ==\n'
output=$(run tftio-bsky-comment-extractor alice.bsky.social 2>&1 || true)
assert_contains "$output" 'BSKY_APP_PASSWORD not set' 'bce handle invocation should reach runtime validation, not clap usage failure'
assert_not_contains "$output" 'Usage:' 'bce handle invocation should preserve extraction syntax'

printf '== unvenv base commands still visible ==\n'
output=$(run tftio-unvenv --help 2>&1)
assert_contains "$output" 'scan         Scan for unignored Python virtual environments (default)' 'unvenv should preserve scan as the default flow'
assert_contains "$output" 'doctor' 'unvenv should expose doctor through the shared base UX'
assert_contains "$output" 'update' 'unvenv should expose update through the shared base UX'

printf '== asana-cli keeps domain command tree ==\n'
output=$(run tftio-asana-cli --help 2>&1)
assert_contains "$output" 'task          Task operations' 'asana-cli should preserve task commands'
assert_contains "$output" 'project       Project operations' 'asana-cli should preserve project commands'
assert_contains "$output" 'doctor' 'asana-cli should expose doctor through the shared base UX'

printf '== bce shared metadata commands available ==\n'
output=$(run tftio-bsky-comment-extractor --help 2>&1)
assert_contains "$output" 'version' 'bce should expose version'
assert_contains "$output" 'license' 'bce should expose license'
assert_contains "$output" 'completions' 'bce should expose completions'
assert_contains "$output" 'doctor' 'bce should expose doctor'

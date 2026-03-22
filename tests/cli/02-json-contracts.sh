#!/bin/sh
set -eu

script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
# shellcheck source=tests/cli/lib.sh
. "${script_dir}/lib.sh"

printf '== shared success contracts ==\n'
output=$(run tftio-todoer list --all --json 2>/dev/null || true)
assert_contains "$output" '"ok":true' 'todoer list should emit ok=true JSON'
assert_contains "$output" '"command":"list"' 'todoer list should emit command=list'

output=$(run tftio-silent-critic --json project init --name consistency-check 2>/dev/null || true)
assert_contains "$output" '"ok":true' 'silent-critic project init should emit ok=true JSON'
assert_contains "$output" '"command":"project.init"' 'silent-critic project init should emit command=project.init'

temp_home=$(mktemp -d)
run_with_home "$temp_home" tftio-silent-critic --json project init --name consistency-check >/dev/null 2>&1
output=$(run_with_home "$temp_home" tftio-silent-critic --json criterion list 2>/dev/null || true)
assert_contains "$output" '"ok":true' 'silent-critic criterion list should emit ok=true JSON'
assert_contains "$output" '"command":"criterion.list"' 'silent-critic criterion list should emit command=criterion.list'
assert_contains "$output" '"criteria":[' 'silent-critic criterion list should emit a criteria array'

printf '== shared error contracts ==\n'
output=$(run tftio-gator --json 2>/dev/null || true)
assert_contains "$output" '"ok":false' 'gator validation failure should emit ok=false JSON'
assert_contains "$output" '"command":"error"' 'gator validation failure should emit command=error'
assert_contains "$output" '"message":"agent is required"' 'gator validation failure should explain the parse error'

output=$(run tftio-todoer task --json show missing-id 2>/dev/null || true)
assert_contains "$output" '"ok":false' 'todoer missing task should emit ok=false JSON'
assert_contains "$output" '"command":"task.show"' 'todoer missing task should emit command=task.show'
assert_contains "$output" '"code":"ERROR"' 'todoer missing task should use the shared error code'

output=$(run tftio-silent-critic --json criterion show missing-id 2>/dev/null || true)
assert_contains "$output" '"ok":false' 'silent-critic missing criterion should emit ok=false JSON'
assert_contains "$output" '"command":"error"' 'silent-critic missing criterion should emit command=error'
assert_contains "$output" '"code":"ERROR"' 'silent-critic missing criterion should use the shared error code'

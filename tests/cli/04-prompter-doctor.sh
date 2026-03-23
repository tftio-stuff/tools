#!/bin/sh
set -eu

script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
# shellcheck source=tests/cli/lib.sh
. "${script_dir}/lib.sh"

printf '== prompter json doctor output ==\n'
output=$(run tftio-prompter doctor --json)
assert_contains "$output" '"config_file_exists":' 'prompter doctor --json should emit config_file_exists'
assert_contains "$output" '"library_directory_exists":' 'prompter doctor --json should emit library_directory_exists'
assert_contains "$output" '"warnings":' 'prompter doctor --json should emit warnings'

printf '== unvenv and asana doctor output ==\n'
output=$(run tftio-unvenv doctor 2>&1)
assert_contains "$output" 'health check' 'unvenv doctor should render a shared health-check header'

temp_home=$(mktemp -d)
output=$(run_with_home "$temp_home" tftio-asana-cli doctor 2>&1)
assert_contains "$output" 'tools health check' 'asana-cli doctor should use the shared doctor renderer'

printf '== prompter completions retain augmentation ==\n'
output=$(run tftio-prompter completions bash)
assert_contains "$output" '__prompter_bash_list_profiles' 'prompter bash completions should keep dynamic profile helpers'
assert_contains "$output" 'source <(prompter completions bash)' 'prompter bash completions should keep the instruction header'

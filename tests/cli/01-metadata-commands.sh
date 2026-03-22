#!/bin/sh
set -eu

script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
# shellcheck source=tests/cli/lib.sh
. "${script_dir}/lib.sh"

printf '== metadata version commands ==\n'
assert_json_version tftio-todoer meta version --json
assert_json_version tftio-gator --json meta version
assert_json_version tftio-silent-critic --json meta version
assert_text_version tftio-bsky-comment-extractor 'bce ' version
assert_text_version tftio-unvenv 'unvenv ' version
assert_text_version tftio-asana-cli 'asana-cli ' version
assert_json_version tftio-prompter version --json

printf '== metadata commands visible in help ==\n'
assert_help_has tftio-todoer meta
assert_help_has tftio-gator meta
assert_help_has tftio-silent-critic meta
assert_help_has tftio-bsky-comment-extractor version
assert_help_has tftio-bsky-comment-extractor doctor
assert_help_has tftio-unvenv update
assert_help_has tftio-asana-cli completions
assert_help_has tftio-prompter doctor

printf '== completion generation ==\n'
assert_completion_has tftio-todoer _todoer meta completions bash
assert_completion_has tftio-bsky-comment-extractor _bce completions bash
assert_completion_has tftio-prompter __prompter_bash_list_profiles completions bash

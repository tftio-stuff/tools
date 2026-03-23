#!/bin/sh
set -eu

script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
repo_root=$(CDPATH= cd -- "${script_dir}/.." && pwd)
test_dir="${repo_root}/tests/cli"

for test_script in "${test_dir}"/[0-9][0-9]-*.sh; do
    printf '\n[%s]\n' "$(basename -- "$test_script")"
    sh "$test_script"
done

printf '\nPASS: repository CLI consistency suite\n'

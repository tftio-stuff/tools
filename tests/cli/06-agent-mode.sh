#!/bin/sh
set -eu

script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
# shellcheck source=tests/cli/lib.sh
. "${script_dir}/lib.sh"

assert_rejected() {
    context=$1
    shift

    stdout_file=$(mktemp)
    stderr_file=$(mktemp)
    if run_agent "$@" >"${stdout_file}" 2>"${stderr_file}"; then
        printf 'FAIL: %s\n' "$context" >&2
        printf 'Command unexpectedly succeeded\n' >&2
        cat "${stdout_file}" >&2
        rm -f "${stdout_file}" "${stderr_file}"
        exit 1
    fi

    stderr=$(cat "${stderr_file}")
    case "$stderr" in
        *"unrecognized subcommand"*|*"unexpected argument"*|*"invalid value"*)
            ;;
        *)
            printf 'FAIL: %s\n' "$context" >&2
            printf 'Unexpected stderr:\n%s\n' "$stderr" >&2
            rm -f "${stdout_file}" "${stderr_file}"
            exit 1
            ;;
    esac

    rm -f "${stdout_file}" "${stderr_file}"
}

printf '== agent-mode help and skill surface ==\n'

bce_help=$(run_agent tftio-bsky-comment-extractor --agent-help)
assert_contains "$bce_help" "query-posts" "bce agent help should expose query-posts"
assert_not_contains "$bce_help" "fetch" "bce agent help should redact fetch"
bce_skill=$(run_agent tftio-bsky-comment-extractor --agent-skill query-posts)
assert_contains "$bce_skill" "capability:" "bce agent skill should include capability section"
assert_contains "$bce_skill" "query-posts" "bce agent skill should render query-posts"
assert_rejected "bce agent mode should reject hidden fetch" tftio-bsky-comment-extractor fetch

gator_help=$(run_agent tftio-gator --agent-help)
assert_contains "$gator_help" "run-agent" "gator agent help should expose run-agent"
assert_not_contains "$gator_help" "meta" "gator agent help should redact meta"
gator_skill=$(run_agent tftio-gator --agent-skill run-agent)
assert_contains "$gator_skill" "capability:" "gator agent skill should include capability section"
assert_contains "$gator_skill" "run-agent" "gator agent skill should render run-agent"
assert_rejected "gator agent mode should reject hidden meta version" tftio-gator meta version

todoer_help=$(run_agent tftio-todoer --agent-help)
assert_contains "$todoer_help" "list-tasks" "todoer agent help should expose list-tasks"
assert_not_contains "$todoer_help" "meta" "todoer agent help should redact meta"
todoer_skill=$(run_agent tftio-todoer --agent-skill list-tasks)
assert_contains "$todoer_skill" "capability:" "todoer agent skill should include capability section"
assert_contains "$todoer_skill" "list-tasks" "todoer agent skill should render list-tasks"
assert_rejected "todoer agent mode should reject hidden meta version" tftio-todoer meta version

unvenv_help=$(run_agent tftio-unvenv --agent-help)
assert_contains "$unvenv_help" "scan-venvs" "unvenv agent help should expose scan-venvs"
assert_not_contains "$unvenv_help" "doctor" "unvenv agent help should redact doctor"
unvenv_skill=$(run_agent tftio-unvenv --agent-skill scan-venvs)
assert_contains "$unvenv_skill" "capability:" "unvenv agent skill should include capability section"
assert_contains "$unvenv_skill" "scan-venvs" "unvenv agent skill should render scan-venvs"
assert_rejected "unvenv agent mode should reject hidden doctor" tftio-unvenv doctor

asana_help=$(run_agent tftio-asana-cli --agent-help)
assert_contains "$asana_help" "manage-tasks" "asana agent help should expose manage-tasks"
assert_not_contains "$asana_help" "doctor" "asana agent help should redact doctor"
asana_skill=$(run_agent tftio-asana-cli --agent-skill manage-tasks)
assert_contains "$asana_skill" "capability:" "asana agent skill should include capability section"
assert_contains "$asana_skill" "manage-tasks" "asana agent skill should render manage-tasks"
assert_rejected "asana agent mode should reject hidden doctor" tftio-asana-cli doctor

critic_help=$(run_agent tftio-silent-critic --agent-help)
assert_contains "$critic_help" "session-manifest" "silent-critic agent help should expose session-manifest"
assert_not_contains "$critic_help" "project" "silent-critic agent help should redact project"
critic_skill=$(run_agent tftio-silent-critic --agent-skill session-manifest)
assert_contains "$critic_skill" "capability:" "silent-critic agent skill should include capability section"
assert_contains "$critic_skill" "session-manifest" "silent-critic agent skill should render session-manifest"
assert_rejected "silent-critic agent mode should reject hidden project init" tftio-silent-critic project init

prompter_help=$(run_agent tftio-prompter --agent-help)
assert_contains "$prompter_help" "render-prompts" "prompter agent help should expose render-prompts"
assert_not_contains "$prompter_help" "doctor" "prompter agent help should redact doctor"
prompter_skill=$(run_agent tftio-prompter --agent-skill render-prompts)
assert_contains "$prompter_skill" "capability:" "prompter agent skill should include capability section"
assert_contains "$prompter_skill" "render-prompts" "prompter agent skill should render render-prompts"
assert_rejected "prompter agent mode should reject hidden doctor" tftio-prompter doctor

printf '== ordinary help and typo redaction ==\n'

redacted_help=$(run_agent tftio-silent-critic --help 2>&1)
assert_contains "$redacted_help" "session" "silent-critic --help should retain visible session commands"
assert_not_contains "$redacted_help" "project" "silent-critic --help should redact hidden project commands"
assert_not_contains "$redacted_help" "criterion" "silent-critic --help should redact hidden criterion commands"

gator_typo_stderr=$(run_agent tftio-gator claude --sessio abc 2>&1 || true)
assert_contains "$gator_typo_stderr" "unexpected argument '--sessio'" "gator hidden flag typo should fail as unexpected argument"
assert_not_contains "$gator_typo_stderr" "--session" "gator hidden flag typo should not reveal hidden flag"
assert_not_contains "$gator_typo_stderr" "Did you mean" "gator hidden flag typo should suppress suggestions"

prompter_typo_stderr=$(run_agent tftio-prompter doctro 2>&1 || true)
assert_contains "$prompter_typo_stderr" "unrecognized subcommand 'doctro'" "prompter hidden command typo should fail as missing command"
assert_not_contains "$prompter_typo_stderr" "doctor" "prompter hidden command typo should not reveal hidden command"
assert_not_contains "$prompter_typo_stderr" "Did you mean" "prompter hidden command typo should suppress suggestions"

printf '\nPASS: agent-mode CLI smoke suite\n'

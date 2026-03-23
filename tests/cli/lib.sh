#!/bin/sh
set -eu

run() {
    package=$1
    shift
    cargo run -q -p "$package" -- "$@"
}

run_agent() {
    package=$1
    shift
    TFTIO_AGENT_TOKEN=phase7-test-token \
    TFTIO_AGENT_TOKEN_EXPECTED=phase7-test-token \
        cargo run -q -p "$package" -- "$@"
}

run_with_home() {
    home=$1
    package=$2
    shift 2
    original_home=$HOME
    CARGO_HOME="${CARGO_HOME:-${original_home}/.cargo}" \
    RUSTUP_HOME="${RUSTUP_HOME:-${original_home}/.rustup}" \
    HOME="$home" \
        cargo run -q -p "$package" -- "$@"
}

assert_contains() {
    haystack=$1
    needle=$2
    context=$3
    printf '%s' "$haystack" | grep -F -- "$needle" >/dev/null || {
        printf 'FAIL: %s\n' "$context" >&2
        printf 'Expected to find: %s\n' "$needle" >&2
        exit 1
    }
}

assert_not_contains() {
    haystack=$1
    needle=$2
    context=$3
    if printf '%s' "$haystack" | grep -F -- "$needle" >/dev/null; then
        printf 'FAIL: %s\n' "$context" >&2
        printf 'Did not expect to find: %s\n' "$needle" >&2
        exit 1
    fi
}

assert_nonempty() {
    value=$1
    context=$2
    [ -n "$value" ] || {
        printf 'FAIL: %s\n' "$context" >&2
        exit 1
    }
}

assert_json_version() {
    package=$1
    shift
    output=$(run "$package" "$@")
    assert_contains "$output" '{"version":"' "${package} should emit JSON version output"
}

assert_text_version() {
    package=$1
    expected_prefix=$2
    shift 2
    output=$(run "$package" "$@")
    assert_contains "$output" "$expected_prefix" "${package} should emit text version output"
}

assert_help_has() {
    package=$1
    needle=$2
    output=$(run "$package" --help 2>&1)
    assert_contains "$output" "$needle" "${package} --help should mention ${needle}"
}

assert_completion_has() {
    package=$1
    needle=$2
    shift 2
    output=$(run "$package" "$@")
    assert_nonempty "$output" "${package} completion output should be non-empty"
    assert_contains "$output" "$needle" "${package} completion output should mention ${needle}"
}

assert_file_not_contains() {
    file=$1
    needle=$2
    context=$3
    if grep -F -- "$needle" "$file" >/dev/null; then
        printf 'FAIL: %s\n' "$context" >&2
        printf 'Did not expect %s to contain: %s\n' "$file" "$needle" >&2
        exit 1
    fi
}

assert_file_contains() {
    file=$1
    needle=$2
    context=$3
    grep -F -- "$needle" "$file" >/dev/null || {
        printf 'FAIL: %s\n' "$context" >&2
        printf 'Expected %s to contain: %s\n' "$file" "$needle" >&2
        exit 1
    }
}

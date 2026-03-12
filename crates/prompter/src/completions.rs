//! Shell completion generation with dynamic profile suggestions.
//!
//! This module wraps `clap_complete` output and augments it so that the
//! `prompter run` subcommand (and the top-level shorthand) offer dynamic
//! profile completions sourced from the active configuration.

use clap::CommandFactory;
use clap_complete::Shell;
use std::io::{self, Write};

use crate::Cli;

/// Generate shell completion script for the requested shell and write it to stdout.
///
/// # Panics
/// Panics if the generated completion script is not valid UTF-8 or if writing to `stdout` fails.
pub fn generate(shell: Shell) {
    let mut cmd = Cli::command();
    let bin_name = cmd.get_name().to_string();

    let instructions = render_instructions(shell, &bin_name);
    let mut buffer = Vec::new();
    clap_complete::generate(shell, &mut cmd, bin_name, &mut buffer);
    let mut script = String::from_utf8(buffer).expect("clap_complete output must be valid UTF-8");

    match shell {
        Shell::Bash => augment_bash(&mut script),
        Shell::Zsh => augment_zsh(&mut script),
        Shell::Fish => augment_fish(&mut script),
        _ => {}
    }

    let mut stdout = io::stdout();
    stdout
        .write_all(instructions.as_bytes())
        .expect("failed to write completion instructions");
    stdout
        .write_all(script.as_bytes())
        .expect("failed to write completion script");
}

fn render_instructions(shell: Shell, bin_name: &str) -> String {
    match shell {
        Shell::Bash => format!(
            "# Shell completion for {bin_name}\n#\n# To enable completions, add this to your shell config:\n#\n#   source <({bin_name} completions bash)\n\n"
        ),
        Shell::Zsh => format!(
            "# Shell completion for {bin_name}\n#\n# To enable completions, add this to your shell config:\n#\n#   {bin_name} completions zsh > ~/.zsh/completions/_{bin_name}\n#   Ensure fpath includes ~/.zsh/completions\n\n"
        ),
        Shell::Fish => format!(
            "# Shell completion for {bin_name}\n#\n# To enable completions, add this to your shell config:\n#\n#   {bin_name} completions fish | source\n\n"
        ),
        Shell::PowerShell => format!(
            "# Shell completion for {bin_name}\n#\n# To enable completions, add this to your shell config:\n#\n#   {bin_name} completions powershell | Out-String | Invoke-Expression\n\n"
        ),
        Shell::Elvish => format!(
            "# Shell completion for {bin_name}\n#\n# To enable completions, add this to your shell config:\n#\n#   {bin_name} completions elvish | eval\n\n"
        ),
        other => format!(
            "# Shell completion for {bin_name}\n#\n# To enable completions, add this to your shell config:\n#\n#   {bin_name} completions {other}\n\n"
        ),
    }
}

fn augment_bash(script: &mut String) {
    const ROOT_REPLACEMENT: &str = r#"        prompter)
            opts="-s -p -P -c -h -V --separator --pre-prompt --post-prompt --config --help --version version license init list validate run completions doctor update help"
            if [[ ${cur} == -* ]]; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --config|-c)
                    COMPREPLY=( $(compgen -f -- "${cur}") )
                    return 0
                    ;;
                --separator|-s|--pre-prompt|-p|--post-prompt|-P)
                    return 0
                    ;;
            esac
            local profiles="$(__prompter_bash_list_profiles)"
            if [[ -n ${profiles} ]]; then
                COMPREPLY=( $(compgen -W "${opts} ${profiles}" -- "${cur}") )
            else
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            fi
            return 0
            ;;"#;

    const RUN_REPLACEMENT: &str = r#"        prompter__run)
            opts="-s -p -P -c -h --separator --pre-prompt --post-prompt --config --help"
            if [[ ${cur} == -* ]]; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --config|-c)
                    COMPREPLY=( $(compgen -f -- "${cur}") )
                    return 0
                    ;;
                --separator|-s|--pre-prompt|-p|--post-prompt|-P)
                    return 0
                    ;;
            esac
            local profiles="$(__prompter_bash_list_profiles)"
            if [[ -n ${profiles} ]]; then
                COMPREPLY=( $(compgen -W "${profiles}" -- "${cur}") )
            fi
            return 0
            ;;"#;

    replace_case_block(script, "prompter", ROOT_REPLACEMENT);
    replace_case_block(script, "prompter__run", RUN_REPLACEMENT);

    script.push_str(BASH_HELPERS);
}

fn augment_zsh(script: &mut String) {
    // With Vec<String>, clap generates '*::profiles' variadic patterns
    const ROOT_MARKER: &str =
        "::profile -- Profile to render (shorthand for 'run `<profile>`'):_default";
    const RUN_MARKER_VARIADIC: &str = "*::profiles -- Profile name(s) to render:_default";

    // Update root shorthand profile completion
    if let Some(start) = script.find(ROOT_MARKER) {
        script.replace_range(
            start..start + ROOT_MARKER.len(),
            "::profile -- Profile to render (shorthand for 'run `<profile>`'):_prompter_dynamic_profiles",
        );
    }

    // Update run subcommand profiles completion (variadic)
    if let Some(start) = script.find(RUN_MARKER_VARIADIC) {
        script.replace_range(
            start..start + RUN_MARKER_VARIADIC.len(),
            "*::profiles -- Profile name(s) to render:_prompter_dynamic_profiles",
        );
    }

    script.push_str(ZSH_HELPERS);
}

fn augment_fish(script: &mut String) {
    script.push_str(FISH_HELPERS);
}

fn replace_case_block(script: &mut String, label: &str, replacement: &str) {
    let pattern = format!("        {label})");
    let start = script
        .find(&pattern)
        .unwrap_or_else(|| panic!("expected case block for {label}"));
    let tail = &script[start..];
    let end_offset = tail
        .find("\n            ;;\n")
        .unwrap_or_else(|| panic!("expected terminator for {label} block"));
    let end = start + end_offset + "\n            ;;\n".len();
    script.replace_range(start..end, replacement);
}

const BASH_HELPERS: &str = r#"
# Dynamic profile helpers appended by prompter.
__prompter_bash_config_value() {
    local idx=1
    local total=${#COMP_WORDS[@]}
    while [[ ${idx} -lt ${total} ]]; do
        case "${COMP_WORDS[idx]}" in
            --config|-c)
                if [[ $((idx + 1)) -lt ${total} ]]; then
                    echo "${COMP_WORDS[idx+1]}"
                fi
                return
                ;;
        esac
        ((idx++))
    done
}

__prompter_bash_list_profiles() {
    local cfg="$(__prompter_bash_config_value)"
    if [[ -n "${cfg}" ]]; then
        prompter list --config "${cfg}" 2>/dev/null
    else
        prompter list 2>/dev/null
    fi
}
"#;

const ZSH_HELPERS: &str = r#"
_prompter_config_value() {
    local idx=1
    local count=$#words
    while (( idx <= count )); do
        case ${words[idx]} in
            --config|-c)
                (( idx++ ))
                if (( idx <= count )); then
                    echo ${words[idx]}
                fi
                return
                ;;
        esac
        (( idx++ ))
    done
}

_prompter_dynamic_profiles() {
    local cfg=$(_prompter_config_value)
    local -a profiles
    if [[ -n ${cfg} ]]; then
        profiles=(${(f)"$(prompter list --config ${cfg:q} 2>/dev/null)"})
    else
        profiles=(${(f)"$(prompter list 2>/dev/null)"})
    fi
    if (( ${#profiles} )); then
        compadd -a profiles
        return 0
    fi
    return 1
}
"#;

const FISH_HELPERS: &str = r#"
function __fish_prompter__config_arg
	set -l tokens (commandline -opc)
	set -e tokens[1]
	for idx in (seq (count $tokens))
		switch $tokens[$idx]
			case '--config'
				set -l next (math $idx + 1)
				if test $next -le (count $tokens)
					echo $tokens[$next]
				end
				return
			case '-c'
				set -l next (math $idx + 1)
				if test $next -le (count $tokens)
					echo $tokens[$next]
				end
				return
		end
	end
end

function __fish_prompter__profiles
	set -l cfg (__fish_prompter__config_arg)
	if test -n "$cfg"
		prompter list --config "$cfg" 2>/dev/null
	else
		prompter list 2>/dev/null
	end
end

complete -c prompter -n "__fish_prompter_needs_command" -f -a "(__fish_prompter__profiles)" -d 'Profile'
complete -c prompter -n "__fish_prompter_using_subcommand run" -f -a "(__fish_prompter__profiles)" -d 'Profile'
"#;

#[cfg(test)]
mod tests {
    use super::*;

    fn raw_script(shell: Shell) -> String {
        let mut cmd = Cli::command();
        let bin_name = cmd.get_name().to_string();
        let mut buf = Vec::new();
        clap_complete::generate(shell, &mut cmd, bin_name, &mut buf);
        String::from_utf8(buf).expect("clap_complete output must be utf-8")
    }

    #[test]
    fn bash_augmentation_injects_dynamic_helpers() {
        let mut script = raw_script(Shell::Bash);
        augment_bash(&mut script);
        assert!(script.contains("__prompter_bash_list_profiles"));
        assert!(script.contains("prompter list --config"));
        assert!(
            !script.contains("[PROFILE]"),
            "static placeholder should be removed in favor of dynamic completion"
        );
    }

    #[test]
    fn zsh_augmentation_redirects_profile_completion() {
        let mut script = raw_script(Shell::Zsh);
        augment_zsh(&mut script);

        // Verify the dynamic profile completion function is present
        assert!(script.contains("_prompter_dynamic_profiles"));
        // Verify it's being used for both shorthand and run subcommand
        assert!(script.contains(":_prompter_dynamic_profiles"));
        // With Vec<String>, the run command should use variadic completion
        assert!(
            script.contains("*::profiles -- Profile name(s) to render:_prompter_dynamic_profiles")
        );
    }

    #[test]
    fn fish_augmentation_appends_profile_commands() {
        let mut script = raw_script(Shell::Fish);
        augment_fish(&mut script);
        assert!(script.contains("__fish_prompter__profiles"));
        assert!(script.contains("prompter list --config"));
    }
}

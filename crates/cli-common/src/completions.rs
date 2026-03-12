//! Shell completion generation module.
//!
//! This module provides generic shell completion generation for CLI tools using clap.
//! It works with any clap `CommandFactory` and generates completions for all major shells.

use clap::CommandFactory;
use clap_complete::Shell;
use std::io;

/// Generate shell completion scripts for a clap-based CLI.
///
/// This function generates shell completions and prints both installation instructions
/// and the completion script to stdout. It supports bash, zsh, fish, elvish, and `PowerShell`.
///
/// # Type Parameters
/// * `T` - A type that implements `CommandFactory` (typically your clap `Cli` struct)
///
/// # Arguments
/// * `shell` - The shell type to generate completions for
///
/// # Examples
/// ```no_run
/// use clap::Parser;
/// use tftio_cli_common::completions::generate_completions;
///
/// #[derive(Parser)]
/// struct Cli {
///     // your CLI definition
/// }
///
/// generate_completions::<Cli>(clap_complete::Shell::Bash);
/// ```
pub fn generate_completions<T: CommandFactory>(shell: Shell) {
    let mut cmd = T::command();
    let bin_name = cmd.get_name().to_string();

    // Print instructions
    println!("# Shell completion for {bin_name}");
    println!("#");
    println!("# To enable completions, add this to your shell config:");
    println!("#");

    match shell {
        Shell::Bash => {
            println!("# For bash (~/.bashrc):");
            println!("#   source <({bin_name} completions bash)");
        }
        Shell::Zsh => {
            println!("# For zsh (~/.zshrc):");
            println!("#   {bin_name} completions zsh > ~/.zsh/completions/_{bin_name}");
            println!("#   # Ensure fpath includes ~/.zsh/completions");
        }
        Shell::Fish => {
            println!("# For fish (~/.config/fish/config.fish):");
            println!("#   {bin_name} completions fish | source");
        }
        _ => {
            println!("# For {shell}:");
            println!("#   {bin_name} completions {shell} > /path/to/completions/_{bin_name}");
        }
    }

    println!();

    // Generate completions
    clap_complete::generate(shell, &mut cmd, bin_name, &mut io::stdout());
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::{Parser, Subcommand};

    #[derive(Parser)]
    #[command(name = "test-cli")]
    struct TestCli {
        #[command(subcommand)]
        command: TestCommands,
    }

    #[derive(Subcommand)]
    enum TestCommands {
        Version,
        Test { arg: String },
    }

    #[test]
    fn test_generate_completions_bash() {
        // Just verify it doesn't panic
        generate_completions::<TestCli>(Shell::Bash);
    }

    #[test]
    fn test_generate_completions_zsh() {
        generate_completions::<TestCli>(Shell::Zsh);
    }

    #[test]
    fn test_generate_completions_fish() {
        generate_completions::<TestCli>(Shell::Fish);
    }

    #[test]
    fn test_generate_completions_elvish() {
        generate_completions::<TestCli>(Shell::Elvish);
    }

    #[test]
    fn test_generate_completions_powershell() {
        generate_completions::<TestCli>(Shell::PowerShell);
    }

    #[test]
    fn test_all_shells_generate_without_panic() {
        let shells = vec![
            Shell::Bash,
            Shell::Zsh,
            Shell::Fish,
            Shell::Elvish,
            Shell::PowerShell,
        ];

        for shell in shells {
            generate_completions::<TestCli>(shell);
        }
    }
}

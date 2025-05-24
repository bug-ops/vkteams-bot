//! Shell completion generation for VK Teams Bot CLI
//!
//! This module provides functionality to generate shell completion scripts
//! for various shells including bash, zsh, fish, and PowerShell.

#[cfg(feature = "completion")]
use crate::cli::Cli;
#[cfg(feature = "completion")]
use crate::errors::prelude::{CliError, Result as CliResult};
#[cfg(feature = "completion")]
use clap::CommandFactory;
#[cfg(feature = "completion")]
use clap_complete::{Shell, generate};
#[cfg(feature = "completion")]
use std::fs;
#[cfg(feature = "completion")]
use std::io;
#[cfg(feature = "completion")]
use std::path::Path;

/// Available shell types for completion generation
#[cfg(feature = "completion")]
#[derive(Debug, Clone, Copy, clap::ValueEnum)]
pub enum CompletionShell {
    Bash,
    Zsh,
    Fish,
    PowerShell,
}

#[cfg(feature = "completion")]
impl From<CompletionShell> for Shell {
    fn from(shell: CompletionShell) -> Self {
        match shell {
            CompletionShell::Bash => Shell::Bash,
            CompletionShell::Zsh => Shell::Zsh,
            CompletionShell::Fish => Shell::Fish,
            CompletionShell::PowerShell => Shell::PowerShell,
        }
    }
}

/// Generate shell completion script for the specified shell
///
/// # Arguments
/// * `shell` - The shell type to generate completions for
/// * `output_path` - Optional path to write the completion script to
///
/// # Returns
/// * `Ok(())` if completion generation succeeds
/// * `Err(CliError)` if generation fails
#[cfg(feature = "completion")]
pub fn generate_completion(shell: CompletionShell, output_path: Option<&Path>) -> CliResult<()> {
    let mut cmd = Cli::command();
    let shell: Shell = shell.into();

    match output_path {
        Some(path) => {
            let mut file = fs::File::create(path).map_err(|e| {
                CliError::FileError(format!("Failed to create completion file: {}", e))
            })?;

            generate(shell, &mut cmd, "vkteams-bot-cli", &mut file);

            println!("Completion script generated: {}", path.display());
            print_installation_instructions(shell, path);
        }
        None => {
            let mut stdout = io::stdout();
            generate(shell, &mut cmd, "vkteams-bot-cli", &mut stdout);
        }
    }

    Ok(())
}

/// Print installation instructions for the generated completion script
#[cfg(feature = "completion")]
fn print_installation_instructions(shell: Shell, path: &Path) {
    println!("\nInstallation Instructions:");
    println!("{}", "=".repeat(50));
    match shell {
        Shell::Bash => {
            println!("Add the following line to your ~/.bashrc or ~/.bash_profile:");
            println!("  source {}", path.display());
            println!("\nOr copy the file to your bash completions directory:");
            println!("  sudo cp {} /etc/bash_completion.d/", path.display());
        }
        Shell::Zsh => {
            println!("Add the following line to your ~/.zshrc:");
            println!("  source {}", path.display());
            println!("\nOr place the file in your zsh completions directory:");
            println!(
                "  cp {} ~/.oh-my-zsh/completions/_vkteams-bot-cli",
                path.display()
            );
            println!("  # or");
            println!(
                "  cp {} /usr/local/share/zsh/site-functions/_vkteams-bot-cli",
                path.display()
            );
        }
        Shell::Fish => {
            println!("Copy the file to your fish completions directory:");
            println!("  cp {} ~/.config/fish/completions/", path.display());
            println!("\nOr for system-wide installation:");
            println!("  sudo cp {} /usr/share/fish/completions/", path.display());
        }
        Shell::PowerShell => {
            println!("Add the following line to your PowerShell profile:");
            println!("  . {}", path.display());
            println!("\nTo find your profile location, run:");
            println!("  $PROFILE");
        }
        _ => {
            println!("Please refer to your shell's documentation for completion installation.");
        }
    }
    println!("\nAfter installation, restart your shell or source the file to enable completions.");
}

/// Generate completion for all supported shells
///
/// # Arguments
/// * `output_dir` - Directory to write completion scripts to
///
/// # Returns
/// * `Ok(())` if all completions are generated successfully
/// * `Err(CliError)` if any generation fails
#[cfg(feature = "completion")]
pub fn generate_all_completions(output_dir: &Path) -> CliResult<()> {
    // Ensure output directory exists
    fs::create_dir_all(output_dir)
        .map_err(|e| CliError::FileError(format!("Failed to create output directory: {}", e)))?;
    let shells = [
        (CompletionShell::Bash, "vkteams-bot-cli.bash"),
        (CompletionShell::Zsh, "_vkteams-bot-cli"),
        (CompletionShell::Fish, "vkteams-bot-cli.fish"),
        (CompletionShell::PowerShell, "vkteams-bot-cli.ps1"),
    ];
    for (shell, filename) in &shells {
        let output_path = output_dir.join(filename);
        generate_completion(*shell, Some(&output_path))?;
    }

    println!(
        "\nAll completion scripts generated in: {}",
        output_dir.display()
    );

    Ok(())
}

/// Get the default completion directory for the current system
#[cfg(feature = "completion")]
pub fn get_default_completion_dir() -> Option<std::path::PathBuf> {
    dirs::home_dir().map(|home| {
        home.join(".local")
            .join("share")
            .join("vkteams-bot-cli")
            .join("completions")
    })
}

/// Install completion script to the appropriate system location
///
/// # Arguments
/// * `shell` - The shell to install completion for
///
/// # Returns
/// * `Ok(())` if installation succeeds
/// * `Err(CliError)` if installation fails
#[cfg(feature = "completion")]
pub fn install_completion(shell: CompletionShell) -> CliResult<()> {
    let temp_dir = std::env::temp_dir();
    let temp_file = temp_dir.join(format!("vkteams-bot-cli-completion-{:?}", shell));

    // Generate completion to temporary file
    generate_completion(shell, Some(&temp_file))?;

    // Determine target location
    let target_path = get_system_completion_path(shell)?;

    // Ensure target directory exists
    if let Some(parent) = target_path.parent() {
        fs::create_dir_all(parent).map_err(|e| {
            CliError::FileError(format!("Failed to create completion directory: {}", e))
        })?;
    }

    // Copy to target location
    fs::copy(&temp_file, &target_path)
        .map_err(|e| CliError::FileError(format!("Failed to install completion: {}", e)))?;

    // Clean up temporary file
    let _ = fs::remove_file(&temp_file);

    println!("Completion installed to: {}", target_path.display());
    print_post_install_instructions(shell);

    Ok(())
}

/// Get the system completion path for a given shell
#[cfg(feature = "completion")]
fn get_system_completion_path(shell: CompletionShell) -> CliResult<std::path::PathBuf> {
    let home = dirs::home_dir()
        .ok_or_else(|| CliError::FileError("Could not determine home directory".to_string()))?;

    let path = match shell {
        CompletionShell::Bash => {
            home.join(".local/share/bash-completion/completions/vkteams-bot-cli")
        }
        CompletionShell::Zsh => home.join(".local/share/zsh/site-functions/_vkteams-bot-cli"),
        CompletionShell::Fish => home.join(".config/fish/completions/vkteams-bot-cli.fish"),
        CompletionShell::PowerShell => {
            // On Windows, use Documents/PowerShell/Scripts
            #[cfg(windows)]
            {
                home.join("Documents/PowerShell/Scripts/vkteams-bot-cli-completion.ps1")
            }
            #[cfg(not(windows))]
            {
                home.join(".config/powershell/Scripts/vkteams-bot-cli-completion.ps1")
            }
        }
    };

    Ok(path)
}

/// Print post-installation instructions
#[cfg(feature = "completion")]
fn print_post_install_instructions(shell: CompletionShell) {
    println!("\nPost-installation steps:");
    match shell {
        CompletionShell::Bash => {
            println!("Add this to your ~/.bashrc if not already present:");
            println!("  eval \"$(register-python-argcomplete vkteams-bot-cli)\"");
            println!("Or restart your terminal to load the new completions.");
        }
        CompletionShell::Zsh => {
            println!("Ensure your zsh completion system is enabled in ~/.zshrc:");
            println!("  autoload -Uz compinit && compinit");
            println!("Then restart your terminal or run: compinit");
        }
        CompletionShell::Fish => {
            println!("Fish will automatically load the completions.");
            println!("Restart your fish shell or run: fish_update_completions");
        }
        CompletionShell::PowerShell => {
            println!("Add this to your PowerShell profile:");
            println!("  Import-Module vkteams-bot-cli-completion");
            println!("Run 'notepad $PROFILE' to edit your profile.");
        }
    }
}

#[cfg(all(test, feature = "completion"))]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_generate_completion_to_stdout() {
        // Test that generation doesn't panic
        assert!(generate_completion(CompletionShell::Bash, None).is_ok());
    }

    #[test]
    fn test_generate_completion_to_file() {
        let temp_dir = tempdir().unwrap();
        let output_path = temp_dir.path().join("test_completion.bash");
        assert!(generate_completion(CompletionShell::Bash, Some(&output_path)).is_ok());
        assert!(output_path.exists());
    }

    #[test]
    fn test_generate_all_completions() {
        let temp_dir = tempdir().unwrap();

        assert!(generate_all_completions(temp_dir.path()).is_ok());

        // Check that files were created
        assert!(temp_dir.path().join("vkteams-bot-cli.bash").exists());
        assert!(temp_dir.path().join("_vkteams-bot-cli").exists());
        assert!(temp_dir.path().join("vkteams-bot-cli.fish").exists());
        assert!(temp_dir.path().join("vkteams-bot-cli.ps1").exists());
    }

    #[test]
    fn test_get_default_completion_dir() {
        let dir = get_default_completion_dir();
        assert!(dir.is_some());
    }

    #[test]
    fn test_shell_conversion() {
        let bash: Shell = CompletionShell::Bash.into();
        assert!(matches!(bash, Shell::Bash));

        let zsh: Shell = CompletionShell::Zsh.into();
        assert!(matches!(zsh, Shell::Zsh));
    }
}

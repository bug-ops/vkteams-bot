//! Shell completion generation for VK Teams Bot CLI
//!
//! This module provides functionality to generate shell completion scripts
//! for various shells including bash, zsh, fish, and PowerShell.

#[cfg(feature = "completion")]
use crate::errors::prelude::{CliError, Result as CliResult};
#[cfg(feature = "completion")]
use clap::Command;
#[cfg(feature = "completion")]
use clap_complete::{generate, Shell};
#[cfg(feature = "completion")]
use std::io;
#[cfg(feature = "completion")]
use std::path::Path;
#[cfg(feature = "completion")]
use std::fs;

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
    // First try to use prebuilt completions if available
    if let Some(prebuilt_path) = get_prebuilt_completion_path(shell) {
        return use_prebuilt_completion(&prebuilt_path, output_path);
    }
    
    // Fall back to generating completions from scratch
    let mut cmd = create_cli_command();
    let shell: Shell = shell.into();
    
    // Set up the command for completion generation
    cmd = enhance_command_for_completion(cmd);
    
    match output_path {
        Some(path) => {
            let mut file = fs::File::create(path)
                .map_err(|e| CliError::FileError(format!("Failed to create completion file: {}", e)))?;
            
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

/// Enhance the command with custom completion logic
#[cfg(feature = "completion")]
fn enhance_command_for_completion(mut cmd: Command) -> Command {
    // Add custom value hints for better completion
    cmd = add_value_hints(cmd);
    
    // Add examples and additional help text
    cmd = add_completion_examples(cmd);
    
    cmd
}

/// Create a CLI command structure for completion generation
#[cfg(feature = "completion")]
fn create_cli_command() -> Command {
    Command::new("vkteams-bot-cli")
        .version("0.6.0")
        .about("VK Teams Bot CLI tool")
        .long_about("A powerful command-line interface for interacting with VK Teams Bot API")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(
            Command::new("send-text")
                .about("Send text message to user or chat")
                .arg(clap::Arg::new("chat_id")
                    .short('u')
                    .long("chat-id")
                    .required(true)
                    .value_name("CHAT_ID")
                    .value_hint(clap::ValueHint::Username))
                .arg(clap::Arg::new("message")
                    .short('m')
                    .long("message")
                    .required(true)
                    .value_name("MESSAGE"))
        )
        .subcommand(
            Command::new("send-file")
                .about("Send file to user or chat")
                .arg(clap::Arg::new("chat_id")
                    .short('u')
                    .long("chat-id")
                    .required(true)
                    .value_name("CHAT_ID")
                    .value_hint(clap::ValueHint::Username))
                .arg(clap::Arg::new("file_path")
                    .short('p')
                    .long("file-path")
                    .required(true)
                    .value_name("FILE_PATH")
                    .value_hint(clap::ValueHint::FilePath))
        )
        .subcommand(
            Command::new("send-voice")
                .about("Send voice message to user or chat")
                .arg(clap::Arg::new("chat_id")
                    .short('u')
                    .long("chat-id")
                    .required(true)
                    .value_name("CHAT_ID")
                    .value_hint(clap::ValueHint::Username))
                .arg(clap::Arg::new("file_path")
                    .short('p')
                    .long("file-path")
                    .required(true)
                    .value_name("FILE_PATH")
                    .value_hint(clap::ValueHint::FilePath))
        )
        .subcommand(
            Command::new("get-chat-info")
                .about("Get chat information")
                .arg(clap::Arg::new("chat_id")
                    .short('c')
                    .long("chat-id")
                    .required(true)
                    .value_name("CHAT_ID")
                    .value_hint(clap::ValueHint::Username))
        )
        .subcommand(
            Command::new("schedule")
                .about("Schedule a message to be sent later")
                .subcommand_required(true)
                .subcommand(
                    Command::new("text")
                        .about("Schedule a text message")
                        .arg(clap::Arg::new("chat_id")
                            .short('u')
                            .long("chat-id")
                            .required(true)
                            .value_name("CHAT_ID")
                            .value_hint(clap::ValueHint::Username))
                        .arg(clap::Arg::new("message")
                            .short('m')
                            .long("message")
                            .required(true)
                            .value_name("MESSAGE"))
                        .arg(clap::Arg::new("time")
                            .short('t')
                            .long("time")
                            .value_name("TIME"))
                )
        )
        .subcommand(
            Command::new("scheduler")
                .about("Manage the scheduler service")
                .subcommand_required(true)
                .subcommand(Command::new("start").about("Start the scheduler daemon"))
                .subcommand(Command::new("stop").about("Stop the scheduler daemon"))
                .subcommand(Command::new("status").about("Show scheduler status"))
                .subcommand(Command::new("list").about("List all scheduled tasks"))
        )
        .subcommand(
            Command::new("config")
                .about("Configure the CLI tool")
                .arg(clap::Arg::new("show")
                    .short('s')
                    .long("show")
                    .action(clap::ArgAction::SetTrue)
                    .help("Show current configuration"))
                .arg(clap::Arg::new("wizard")
                    .short('w')
                    .long("wizard")
                    .action(clap::ArgAction::SetTrue)
                    .help("Interactive configuration wizard"))
        )
        .subcommand(
            Command::new("completion")
                .about("Generate shell completion scripts")
                .arg(clap::Arg::new("shell")
                    .required(true)
                    .value_parser(["bash", "zsh", "fish", "powershell"])
                    .help("Shell to generate completion for"))
                .arg(clap::Arg::new("output")
                    .short('o')
                    .long("output")
                    .value_name("PATH")
                    .value_hint(clap::ValueHint::FilePath)
                    .help("Output file path"))
                .arg(clap::Arg::new("install")
                    .short('i')
                    .long("install")
                    .action(clap::ArgAction::SetTrue)
                    .help("Install completion to system location"))
                .arg(clap::Arg::new("all")
                    .short('a')
                    .long("all")
                    .action(clap::ArgAction::SetTrue)
                    .help("Generate completions for all shells"))
        )
        .subcommand(Command::new("setup").about("Interactive setup wizard"))
        .subcommand(Command::new("validate").about("Validate current configuration"))
        .subcommand(Command::new("examples").about("Show usage examples"))
        .subcommand(Command::new("get-self").about("Get bot information"))
}

/// Add value hints to improve completion experience
#[cfg(feature = "completion")]
fn add_value_hints(mut cmd: Command) -> Command {
    // Find and enhance arguments with appropriate value hints
    for subcommand in cmd.get_subcommands_mut() {
        enhance_subcommand_args(subcommand);
    }
    
    cmd
}

/// Enhance subcommand arguments with value hints
#[cfg(feature = "completion")]
fn enhance_subcommand_args(cmd: &mut Command) {
    // Note: In newer versions of clap, direct argument mutation is more complex
    // For now, we'll rely on the value hints set in the derive macros
    
    // Recursively enhance nested subcommands
    for subcommand in cmd.get_subcommands_mut() {
        enhance_subcommand_args(subcommand);
    }
}

/// Add examples to improve completion help
#[cfg(feature = "completion")]
fn add_completion_examples(mut cmd: Command) -> Command {
    let examples = vec![
        "vkteams-bot-cli send-text -u user123 -m \"Hello\"",
        "vkteams-bot-cli send-file -u chat456 -p /path/to/file.pdf",
        "vkteams-bot-cli get-chat-info -c chat789",
        "vkteams-bot-cli schedule text -u user123 -m \"Reminder\" -t \"2024-01-01 10:00\"",
        "vkteams-bot-cli scheduler start",
        "vkteams-bot-cli validate",
    ];
    
    let examples_text = examples.join("\n");
    cmd = cmd.after_help(format!("EXAMPLES:\n{}", examples_text));
    
    cmd
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
            println!("  cp {} ~/.oh-my-zsh/completions/_vkteams-bot-cli", path.display());
            println!("  # or");
            println!("  cp {} /usr/local/share/zsh/site-functions/_vkteams-bot-cli", path.display());
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
    
    // Try to find prebuilt completions in common locations
    if let Some(prebuilt_dir) = find_prebuilt_completions_dir() {
        return copy_all_prebuilt_completions(&prebuilt_dir, output_dir);
    }
    
    // Fall back to generating completions from scratch
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
    
    println!("\nAll completion scripts generated in: {}", output_dir.display());
    
    Ok(())
}

/// Copy all prebuilt completions to the output directory
#[cfg(feature = "completion")]
fn copy_all_prebuilt_completions(prebuilt_dir: &Path, output_dir: &Path) -> CliResult<()> {
    let files = [
        "vkteams-bot-cli.bash",
        "_vkteams-bot-cli", 
        "vkteams-bot-cli.fish",
        "vkteams-bot-cli.ps1",
    ];
    
    let mut copied_count = 0;
    
    for filename in &files {
        let src = prebuilt_dir.join(filename);
        let dst = output_dir.join(filename);
        
        if src.exists() {
            fs::copy(&src, &dst)
                .map_err(|e| CliError::FileError(format!("Failed to copy {}: {}", filename, e)))?;
            copied_count += 1;
            println!("Copied prebuilt completion: {}", dst.display());
        }
    }
    
    if copied_count > 0 {
        println!("\nCopied {} prebuilt completion scripts to: {}", copied_count, output_dir.display());
        Ok(())
    } else {
        Err(CliError::FileError("No prebuilt completions found to copy".to_string()))
    }
}

/// Get the default completion directory for the current system
#[cfg(feature = "completion")]
pub fn get_default_completion_dir() -> Option<std::path::PathBuf> {
    if let Some(home) = dirs::home_dir() {
        Some(home.join(".local").join("share").join("vkteams-bot-cli").join("completions"))
    } else {
        None
    }
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
        fs::create_dir_all(parent)
            .map_err(|e| CliError::FileError(format!("Failed to create completion directory: {}", e)))?;
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
        CompletionShell::Bash => home.join(".local/share/bash-completion/completions/vkteams-bot-cli"),
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

/// Get path to prebuilt completion file for the specified shell
#[cfg(feature = "completion")]
fn get_prebuilt_completion_path(shell: CompletionShell) -> Option<std::path::PathBuf> {
    let prebuilt_dir = find_prebuilt_completions_dir()?;
    
    let filename = match shell {
        CompletionShell::Bash => "vkteams-bot-cli.bash",
        CompletionShell::Zsh => "_vkteams-bot-cli", 
        CompletionShell::Fish => "vkteams-bot-cli.fish",
        CompletionShell::PowerShell => "vkteams-bot-cli.ps1",
    };
    
    let file_path = prebuilt_dir.join(filename);
    if file_path.exists() {
        Some(file_path)
    } else {
        None
    }
}

/// Use prebuilt completion script
#[cfg(feature = "completion")]
fn use_prebuilt_completion(prebuilt_path: &Path, output_path: Option<&Path>) -> CliResult<()> {
    match output_path {
        Some(path) => {
            fs::copy(prebuilt_path, path)
                .map_err(|e| CliError::FileError(format!("Failed to copy prebuilt completion: {}", e)))?;
            
            println!("Prebuilt completion script copied: {}", path.display());
            let shell = shell_from_str(&extract_shell_from_path(prebuilt_path)).unwrap_or(Shell::Bash);
            print_installation_instructions(shell, path);
        }
        None => {
            let content = fs::read_to_string(prebuilt_path)
                .map_err(|e| CliError::FileError(format!("Failed to read prebuilt completion: {}", e)))?;
            print!("{}", content);
        }
    }
    
    Ok(())
}

/// Extract shell name from completion file path
#[cfg(feature = "completion")]
fn extract_shell_from_path(path: &Path) -> String {
    let filename = path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("");
    
    if filename.contains("bash") {
        "bash".to_string()
    } else if filename.starts_with('_') {
        "zsh".to_string() 
    } else if filename.contains("fish") {
        "fish".to_string()
    } else if filename.contains("ps1") {
        "powershell".to_string()
    } else {
        "bash".to_string()
    }
}

/// Find prebuilt completions directory
#[cfg(feature = "completion")]
fn find_prebuilt_completions_dir() -> Option<std::path::PathBuf> {
    // Check if completions directory is available from build time
    if let Some(dir) = option_env!("VKTEAMS_COMPLETIONS_DIR") {
        let path = std::path::PathBuf::from(dir);
        if path.exists() {
            return Some(path);
        }
    }
    
    // Fall back to checking common build locations relative to current executable
    if let Ok(current_exe) = std::env::current_exe() {
        if let Some(exe_dir) = current_exe.parent() {
            let possible_paths = [
                exe_dir.join("completions"),
                exe_dir.join("../completions"),
                exe_dir.join("../../completions"),
                exe_dir.join("../../../completions"), // For nested target dirs
                exe_dir.join("../../../../target/completions"), // From installed location
            ];
            
            for path in &possible_paths {
                if path.exists() && path.is_dir() {
                    return Some(path.canonicalize().unwrap_or_else(|_| path.clone()));
                }
            }
        }
    }
    
    None
}

/// Convert string to Shell enum
#[cfg(feature = "completion")]
fn shell_from_str(s: &str) -> Option<Shell> {
    match s.to_lowercase().as_str() {
        "bash" => Some(Shell::Bash),
        "zsh" => Some(Shell::Zsh),
        "fish" => Some(Shell::Fish),
        "powershell" => Some(Shell::PowerShell),
        _ => None,
    }
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
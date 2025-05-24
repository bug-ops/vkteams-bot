use clap::Command;
use clap_complete::{Shell, generate_to};
use std::env;
use std::io::Error;
use std::path::PathBuf;

fn main() -> Result<(), Error> {
    // Only generate completions in release builds or when explicitly requested
    let should_generate = env::var("CARGO_FEATURE_COMPLETION").is_ok()
        || env::var("VKTEAMS_GENERATE_COMPLETIONS").is_ok()
        || env::var("PROFILE").map_or(false, |p| p == "release");

    if !should_generate {
        println!(
            "cargo:warning=Skipping completion generation (use VKTEAMS_GENERATE_COMPLETIONS=1 to force)"
        );
        return Ok(());
    }

    let outdir = match env::var_os("OUT_DIR") {
        None => return Ok(()),
        Some(outdir) => outdir,
    };

    let mut path = PathBuf::from(outdir);
    path.push("completions");

    // Create completions directory
    std::fs::create_dir_all(&path)?;

    let mut cmd = create_cli_command();

    // Generate completions for all supported shells
    let shells = [
        (Shell::Bash, "vkteams-bot-cli.bash"),
        (Shell::Zsh, "_vkteams-bot-cli"),
        (Shell::Fish, "vkteams-bot-cli.fish"),
        (Shell::PowerShell, "vkteams-bot-cli.ps1"),
    ];

    for (shell, _file_name) in shells.iter() {
        match generate_to(*shell, &mut cmd, "vkteams-bot-cli", &path) {
            Ok(completion_path) => {
                println!(
                    "cargo:warning=Generated {} completion: {:?}",
                    shell_name(*shell),
                    completion_path
                );
            }
            Err(e) => {
                println!(
                    "cargo:warning=Failed to generate {} completion: {}",
                    shell_name(*shell),
                    e
                );
            }
        }
    }

    // Copy completions to a more accessible location
    let target_dir = env::var("CARGO_TARGET_DIR")
        .or_else(|_| env::var("CARGO_MANIFEST_DIR").map(|d| format!("{}/target", d)))
        .unwrap_or_else(|_| "target".to_string());

    let target_completions = format!("{}/completions", target_dir);

    if let Err(e) = std::fs::create_dir_all(&target_completions) {
        println!(
            "cargo:warning=Failed to create target completions directory: {}",
            e
        );
        return Ok(());
    }

    // Copy generated completions to target directory
    for (_, file_name) in shells.iter() {
        let src = path.join(file_name);
        let dst = PathBuf::from(&target_completions).join(file_name);

        if src.exists() {
            if let Err(e) = std::fs::copy(&src, &dst) {
                println!(
                    "cargo:warning=Failed to copy completion file {}: {}",
                    file_name, e
                );
            } else {
                println!("cargo:warning=Copied completion: {:?}", dst);
            }
        }
    }

    // Generate installation script
    generate_install_script(&target_completions)?;

    // Set environment variable for runtime access
    println!(
        "cargo:rustc-env=VKTEAMS_COMPLETIONS_DIR={}",
        target_completions
    );

    // Tell cargo to rerun this script if these files change
    println!("cargo:rerun-if-changed=src/main.rs");
    println!("cargo:rerun-if-changed=src/commands/");
    println!("cargo:rerun-if-changed=Cargo.toml");
    println!("cargo:rerun-if-env-changed=VKTEAMS_GENERATE_COMPLETIONS");

    Ok(())
}

/// Create a simplified CLI command structure for completion generation
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
                .arg(
                    clap::Arg::new("chat_id")
                        .short('u')
                        .long("chat-id")
                        .required(true)
                        .value_name("CHAT_ID")
                        .value_hint(clap::ValueHint::Username),
                )
                .arg(
                    clap::Arg::new("message")
                        .short('m')
                        .long("message")
                        .required(true)
                        .value_name("MESSAGE"),
                ),
        )
        .subcommand(
            Command::new("send-file")
                .about("Send file to user or chat")
                .arg(
                    clap::Arg::new("chat_id")
                        .short('u')
                        .long("chat-id")
                        .required(true)
                        .value_name("CHAT_ID")
                        .value_hint(clap::ValueHint::Username),
                )
                .arg(
                    clap::Arg::new("file_path")
                        .short('p')
                        .long("file-path")
                        .required(true)
                        .value_name("FILE_PATH")
                        .value_hint(clap::ValueHint::FilePath),
                ),
        )
        .subcommand(
            Command::new("send-voice")
                .about("Send voice message to user or chat")
                .arg(
                    clap::Arg::new("chat_id")
                        .short('u')
                        .long("chat-id")
                        .required(true)
                        .value_name("CHAT_ID")
                        .value_hint(clap::ValueHint::Username),
                )
                .arg(
                    clap::Arg::new("file_path")
                        .short('p')
                        .long("file-path")
                        .required(true)
                        .value_name("FILE_PATH")
                        .value_hint(clap::ValueHint::FilePath),
                ),
        )
        .subcommand(
            Command::new("get-chat-info")
                .about("Get chat information")
                .arg(
                    clap::Arg::new("chat_id")
                        .short('c')
                        .long("chat-id")
                        .required(true)
                        .value_name("CHAT_ID")
                        .value_hint(clap::ValueHint::Username),
                ),
        )
        .subcommand(
            Command::new("schedule")
                .about("Schedule a message to be sent later")
                .subcommand_required(true)
                .subcommand(
                    Command::new("text")
                        .about("Schedule a text message")
                        .arg(
                            clap::Arg::new("chat_id")
                                .short('u')
                                .long("chat-id")
                                .required(true)
                                .value_name("CHAT_ID")
                                .value_hint(clap::ValueHint::Username),
                        )
                        .arg(
                            clap::Arg::new("message")
                                .short('m')
                                .long("message")
                                .required(true)
                                .value_name("MESSAGE"),
                        )
                        .arg(
                            clap::Arg::new("time")
                                .short('t')
                                .long("time")
                                .value_name("TIME"),
                        ),
                )
                .subcommand(
                    Command::new("file")
                        .about("Schedule a file message")
                        .arg(
                            clap::Arg::new("chat_id")
                                .short('u')
                                .long("chat-id")
                                .required(true)
                                .value_name("CHAT_ID")
                                .value_hint(clap::ValueHint::Username),
                        )
                        .arg(
                            clap::Arg::new("file_path")
                                .short('p')
                                .long("file-path")
                                .required(true)
                                .value_name("FILE_PATH")
                                .value_hint(clap::ValueHint::FilePath),
                        ),
                )
                .subcommand(
                    Command::new("voice")
                        .about("Schedule a voice message")
                        .arg(
                            clap::Arg::new("chat_id")
                                .short('u')
                                .long("chat-id")
                                .required(true)
                                .value_name("CHAT_ID")
                                .value_hint(clap::ValueHint::Username),
                        )
                        .arg(
                            clap::Arg::new("file_path")
                                .short('p')
                                .long("file-path")
                                .required(true)
                                .value_name("FILE_PATH")
                                .value_hint(clap::ValueHint::FilePath),
                        ),
                )
                .subcommand(
                    Command::new("action")
                        .about("Schedule a chat action")
                        .arg(
                            clap::Arg::new("chat_id")
                                .short('u')
                                .long("chat-id")
                                .required(true)
                                .value_name("CHAT_ID")
                                .value_hint(clap::ValueHint::Username),
                        )
                        .arg(
                            clap::Arg::new("action")
                                .short('a')
                                .long("action")
                                .required(true)
                                .value_name("ACTION"),
                        ),
                ),
        )
        .subcommand(
            Command::new("scheduler")
                .about("Manage the scheduler service")
                .subcommand_required(true)
                .subcommand(Command::new("start").about("Start the scheduler daemon"))
                .subcommand(Command::new("stop").about("Stop the scheduler daemon"))
                .subcommand(Command::new("status").about("Show scheduler status"))
                .subcommand(Command::new("list").about("List all scheduled tasks")),
        )
        .subcommand(
            Command::new("task")
                .about("Manage scheduled tasks")
                .subcommand_required(true)
                .subcommand(
                    Command::new("show")
                        .about("Show details of a specific task")
                        .arg(
                            clap::Arg::new("task_id")
                                .required(true)
                                .value_name("TASK_ID"),
                        ),
                )
                .subcommand(
                    Command::new("remove").about("Remove a scheduled task").arg(
                        clap::Arg::new("task_id")
                            .required(true)
                            .value_name("TASK_ID"),
                    ),
                )
                .subcommand(
                    Command::new("enable").about("Enable a disabled task").arg(
                        clap::Arg::new("task_id")
                            .required(true)
                            .value_name("TASK_ID"),
                    ),
                )
                .subcommand(
                    Command::new("disable").about("Disable an active task").arg(
                        clap::Arg::new("task_id")
                            .required(true)
                            .value_name("TASK_ID"),
                    ),
                )
                .subcommand(
                    Command::new("run").about("Run a task immediately").arg(
                        clap::Arg::new("task_id")
                            .required(true)
                            .value_name("TASK_ID"),
                    ),
                ),
        )
        .subcommand(
            Command::new("config")
                .about("Configure the CLI tool")
                .arg(
                    clap::Arg::new("show")
                        .short('s')
                        .long("show")
                        .action(clap::ArgAction::SetTrue)
                        .help("Show current configuration"),
                )
                .arg(
                    clap::Arg::new("wizard")
                        .short('w')
                        .long("wizard")
                        .action(clap::ArgAction::SetTrue)
                        .help("Interactive configuration wizard"),
                ),
        )
        .subcommand(
            Command::new("completion")
                .about("Generate shell completion scripts")
                .arg(
                    clap::Arg::new("shell")
                        .required(true)
                        .value_parser(["bash", "zsh", "fish", "powershell"])
                        .help("Shell to generate completion for"),
                )
                .arg(
                    clap::Arg::new("output")
                        .short('o')
                        .long("output")
                        .value_name("PATH")
                        .value_hint(clap::ValueHint::FilePath)
                        .help("Output file path"),
                )
                .arg(
                    clap::Arg::new("install")
                        .short('i')
                        .long("install")
                        .action(clap::ArgAction::SetTrue)
                        .help("Install completion to system location"),
                )
                .arg(
                    clap::Arg::new("all")
                        .short('a')
                        .long("all")
                        .action(clap::ArgAction::SetTrue)
                        .help("Generate completions for all shells"),
                ),
        )
        .subcommand(Command::new("setup").about("Interactive setup wizard"))
        .subcommand(Command::new("validate").about("Validate current configuration"))
        .subcommand(Command::new("examples").about("Show usage examples"))
        .subcommand(Command::new("list-commands").about("Show all available commands"))
        .subcommand(
            Command::new("get-self").about("Get bot information").arg(
                clap::Arg::new("detailed")
                    .short('d')
                    .long("detailed")
                    .action(clap::ArgAction::SetTrue)
                    .help("Show detailed information"),
            ),
        )
        .subcommand(
            Command::new("get-events")
                .about("Get events or start long polling")
                .arg(
                    clap::Arg::new("listen")
                        .short('l')
                        .long("listen")
                        .value_parser(clap::value_parser!(bool))
                        .help("Start long polling"),
                ),
        )
        .subcommand(
            Command::new("get-file")
                .about("Download file by ID")
                .arg(
                    clap::Arg::new("file_id")
                        .short('f')
                        .long("file-id")
                        .required(true)
                        .value_name("FILE_ID"),
                )
                .arg(
                    clap::Arg::new("file_path")
                        .short('p')
                        .long("file-path")
                        .value_name("FILE_PATH")
                        .value_hint(clap::ValueHint::DirPath),
                ),
        )
        .subcommand(
            Command::new("edit-message")
                .about("Edit existing message")
                .arg(
                    clap::Arg::new("chat_id")
                        .short('c')
                        .long("chat-id")
                        .required(true)
                        .value_name("CHAT_ID")
                        .value_hint(clap::ValueHint::Username),
                )
                .arg(
                    clap::Arg::new("message_id")
                        .short('m')
                        .long("message-id")
                        .required(true)
                        .value_name("MESSAGE_ID"),
                )
                .arg(
                    clap::Arg::new("new_text")
                        .short('t')
                        .long("text")
                        .required(true)
                        .value_name("NEW_TEXT"),
                ),
        )
        .subcommand(
            Command::new("delete-message")
                .about("Delete message from chat")
                .arg(
                    clap::Arg::new("chat_id")
                        .short('c')
                        .long("chat-id")
                        .required(true)
                        .value_name("CHAT_ID")
                        .value_hint(clap::ValueHint::Username),
                )
                .arg(
                    clap::Arg::new("message_id")
                        .short('m')
                        .long("message-id")
                        .required(true)
                        .value_name("MESSAGE_ID"),
                ),
        )
        .subcommand(
            Command::new("pin-message")
                .about("Pin message in chat")
                .arg(
                    clap::Arg::new("chat_id")
                        .short('c')
                        .long("chat-id")
                        .required(true)
                        .value_name("CHAT_ID")
                        .value_hint(clap::ValueHint::Username),
                )
                .arg(
                    clap::Arg::new("message_id")
                        .short('m')
                        .long("message-id")
                        .required(true)
                        .value_name("MESSAGE_ID"),
                ),
        )
        .subcommand(
            Command::new("unpin-message")
                .about("Unpin message from chat")
                .arg(
                    clap::Arg::new("chat_id")
                        .short('c')
                        .long("chat-id")
                        .required(true)
                        .value_name("CHAT_ID")
                        .value_hint(clap::ValueHint::Username),
                )
                .arg(
                    clap::Arg::new("message_id")
                        .short('m')
                        .long("message-id")
                        .required(true)
                        .value_name("MESSAGE_ID"),
                ),
        )
        .subcommand(
            Command::new("get-profile")
                .about("Get user profile information")
                .arg(
                    clap::Arg::new("user_id")
                        .short('u')
                        .long("user-id")
                        .required(true)
                        .value_name("USER_ID")
                        .value_hint(clap::ValueHint::Username),
                ),
        )
        .subcommand(
            Command::new("get-chat-members")
                .about("Get chat members")
                .arg(
                    clap::Arg::new("chat_id")
                        .short('c')
                        .long("chat-id")
                        .required(true)
                        .value_name("CHAT_ID")
                        .value_hint(clap::ValueHint::Username),
                )
                .arg(
                    clap::Arg::new("cursor")
                        .short('r')
                        .long("cursor")
                        .value_name("CURSOR"),
                ),
        )
        .subcommand(
            Command::new("set-chat-title")
                .about("Set chat title")
                .arg(
                    clap::Arg::new("chat_id")
                        .short('c')
                        .long("chat-id")
                        .required(true)
                        .value_name("CHAT_ID")
                        .value_hint(clap::ValueHint::Username),
                )
                .arg(
                    clap::Arg::new("title")
                        .short('t')
                        .long("title")
                        .required(true)
                        .value_name("TITLE"),
                ),
        )
        .subcommand(
            Command::new("set-chat-about")
                .about("Set chat description")
                .arg(
                    clap::Arg::new("chat_id")
                        .short('c')
                        .long("chat-id")
                        .required(true)
                        .value_name("CHAT_ID")
                        .value_hint(clap::ValueHint::Username),
                )
                .arg(
                    clap::Arg::new("about")
                        .short('a')
                        .long("about")
                        .required(true)
                        .value_name("ABOUT"),
                ),
        )
        .subcommand(
            Command::new("send-action")
                .about("Send typing or looking action to chat")
                .arg(
                    clap::Arg::new("chat_id")
                        .short('c')
                        .long("chat-id")
                        .required(true)
                        .value_name("CHAT_ID")
                        .value_hint(clap::ValueHint::Username),
                )
                .arg(
                    clap::Arg::new("action")
                        .short('a')
                        .long("action")
                        .required(true)
                        .value_name("ACTION")
                        .value_parser(["typing", "looking"]),
                ),
        )
}

fn shell_name(shell: Shell) -> &'static str {
    match shell {
        Shell::Bash => "bash",
        Shell::Zsh => "zsh",
        Shell::Fish => "fish",
        Shell::PowerShell => "powershell",
        _ => "unknown",
    }
}

fn generate_install_script(completions_dir: &str) -> Result<(), Error> {
    let script_content = format!(
        r#"#!/bin/bash
# Auto-generated installation script for VK Teams Bot CLI completions
# Generated during build process

set -e

COMPLETIONS_DIR="{}"
CLI_NAME="vkteams-bot-cli"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

print_info() {{
    echo -e "${{BLUE}}ℹ️  $1${{NC}}"
}}

print_success() {{
    echo -e "${{GREEN}}✅ $1${{NC}}"
}}

print_warning() {{
    echo -e "${{YELLOW}}⚠️  $1${{NC}}"
}}

print_error() {{
    echo -e "${{RED}}❌ $1${{NC}}"
}}

detect_shell() {{
    if [ -n "$SHELL" ]; then
        basename "$SHELL"
    else
        echo "bash"
    fi
}}

install_completion() {{
    local shell="$1"
    local force="$2"

    case "$shell" in
        bash)
            local completion_file="$COMPLETIONS_DIR/vkteams-bot-cli.bash"
            local target_dir="$HOME/.local/share/bash-completion/completions"
            local target_file="$target_dir/$CLI_NAME"

            if [ -f "$completion_file" ]; then
                mkdir -p "$target_dir"
                cp "$completion_file" "$target_file"
                print_success "Bash completion installed to $target_file"

                if ! grep -q "source $target_file" ~/.bashrc 2>/dev/null; then
                    echo "source $target_file" >> ~/.bashrc
                    print_success "Added source line to ~/.bashrc"
                fi
            else
                print_error "Bash completion file not found: $completion_file"
                return 1
            fi
            ;;

        zsh)
            local completion_file="$COMPLETIONS_DIR/_vkteams-bot-cli"
            local target_dir="$HOME/.local/share/zsh/site-functions"
            local target_file="$target_dir/_$CLI_NAME"

            if [ -f "$completion_file" ]; then
                mkdir -p "$target_dir"
                cp "$completion_file" "$target_file"
                print_success "Zsh completion installed to $target_file"

                if ! grep -q "fpath.*$target_dir" ~/.zshrc 2>/dev/null; then
                    echo "fpath=($target_dir \$fpath)" >> ~/.zshrc
                    echo "autoload -Uz compinit && compinit" >> ~/.zshrc
                    print_success "Added completion setup to ~/.zshrc"
                fi
            else
                print_error "Zsh completion file not found: $completion_file"
                return 1
            fi
            ;;

        fish)
            local completion_file="$COMPLETIONS_DIR/vkteams-bot-cli.fish"
            local target_dir="$HOME/.config/fish/completions"
            local target_file="$target_dir/$CLI_NAME.fish"

            if [ -f "$completion_file" ]; then
                mkdir -p "$target_dir"
                cp "$completion_file" "$target_file"
                print_success "Fish completion installed to $target_file"
            else
                print_error "Fish completion file not found: $completion_file"
                return 1
            fi
            ;;

        powershell)
            print_warning "PowerShell completion requires manual installation"
            print_info "Copy $COMPLETIONS_DIR/vkteams-bot-cli.ps1 to your PowerShell profile directory"
            ;;

        *)
            print_error "Unsupported shell: $shell"
            return 1
            ;;
    esac
}}

main() {{
    local target_shell="$1"
    local force="$2"

    print_info "VK Teams Bot CLI Completion Installer (Build-time Generated)"
    echo

    if [ ! -d "$COMPLETIONS_DIR" ]; then
        print_error "Completions directory not found: $COMPLETIONS_DIR"
        print_info "Make sure you built the CLI with completion generation enabled"
        exit 1
    fi

    if [ -z "$target_shell" ]; then
        target_shell=$(detect_shell)
        print_info "Auto-detected shell: $target_shell"
    fi

    install_completion "$target_shell" "$force"

    echo
    print_success "Installation complete!"
    print_info "Restart your shell or source your configuration file to enable completions"
}}

if [ "${{BASH_SOURCE[0]}}" == "${{0}}" ]; then
    main "$@"
fi
"#,
        completions_dir
    );

    let script_path = format!("{}/install-completions.sh", completions_dir);
    std::fs::write(&script_path, script_content)?;

    // Make script executable on Unix systems
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&script_path)?.permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&script_path, perms)?;
    }

    println!(
        "cargo:warning=Generated installation script: {}",
        script_path
    );
    Ok(())
}

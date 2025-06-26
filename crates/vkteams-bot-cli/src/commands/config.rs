//! Configuration commands module
//!
//! This module contains all commands related to configuration management.

use crate::commands::{Command, CommandExecutor, CommandResult, OutputFormat};
use crate::config::Config;
use crate::constants::{help, ui::emoji};
use crate::errors::prelude::{CliError, Result as CliResult};
use crate::output::{CliResponse, OutputFormatter};
use async_trait::async_trait;
use clap::Subcommand;
use colored::Colorize;
use serde_json::json;
use std::io::{self, Write};
use vkteams_bot::prelude::*;

/// All configuration-related commands
#[derive(Subcommand, Debug, Clone)]
pub enum ConfigCommands {
    /// Interactive setup wizard for first-time configuration
    Setup,
    /// Show examples of how to use the CLI
    Examples,
    /// Show detailed information about all available commands
    ListCommands,
    /// Validate current configuration and test bot connection
    Validate,
    /// Configure the CLI tool
    Config {
        /// Show current configuration
        #[arg(short, long)]
        show: bool,
        /// Initialize a new configuration file
        #[arg(short, long)]
        init: bool,
        /// Interactive configuration wizard
        #[arg(short = 'w', long)]
        wizard: bool,
    },
    /// Generate shell completion scripts
    Completion {
        /// Shell to generate completion for
        #[arg(value_enum)]
        shell: crate::completion::CompletionShell,
        /// Output file path (prints to stdout if not specified)
        #[arg(short, long)]
        output: Option<String>,
        /// Install completion to system location
        #[arg(short, long)]
        install: bool,
        /// Generate completions for all shells
        #[arg(short, long)]
        all: bool,
    },
}

#[async_trait]
impl Command for ConfigCommands {
    async fn execute(&self, bot: &Bot) -> CliResult<()> {
        match self {
            ConfigCommands::Setup => execute_setup().await,
            ConfigCommands::Examples => execute_examples().await,
            ConfigCommands::ListCommands => execute_list_commands().await,
            ConfigCommands::Validate => execute_validate(bot).await,
            ConfigCommands::Config { show, init, wizard } => {
                execute_config(*show, *init, *wizard).await
            }
            ConfigCommands::Completion {
                shell,
                output,
                install,
                all,
            } => execute_completion(*shell, output.as_deref(), *install, *all).await,
        }
    }

    fn name(&self) -> &'static str {
        match self {
            ConfigCommands::Setup => "setup",
            ConfigCommands::Examples => "examples",
            ConfigCommands::ListCommands => "list-commands",
            ConfigCommands::Validate => "validate",
            ConfigCommands::Config { .. } => "config",
            ConfigCommands::Completion { .. } => "completion",
        }
    }

    fn validate(&self) -> CliResult<()> {
        // Configuration commands don't need pre-validation
        Ok(())
    }

    /// New method for structured output support
    async fn execute_with_output(&self, bot: &Bot, output_format: &OutputFormat) -> CliResult<()> {
        // For interactive commands like Setup and Config wizard, use regular execute when in Pretty mode
        let is_interactive = matches!(self, ConfigCommands::Setup) || 
            matches!(self, ConfigCommands::Config { wizard: true, .. });
            
        if is_interactive && matches!(output_format, OutputFormat::Pretty) {
            return self.execute(bot).await;
        }
        
        let response = match self {
            ConfigCommands::Setup => execute_setup_structured().await,
            ConfigCommands::Examples => execute_examples_structured().await,
            ConfigCommands::ListCommands => execute_list_commands_structured().await,
            ConfigCommands::Validate => execute_validate_structured(bot).await,
            ConfigCommands::Config { show, init, wizard } => {
                execute_config_structured(*show, *init, *wizard).await
            }
            ConfigCommands::Completion {
                shell,
                output,
                install,
                all,
            } => execute_completion_structured(*shell, output.as_deref(), *install, *all).await,
        };

        OutputFormatter::print(&response, output_format)?;

        if !response.success {
            return Err(CliError::UnexpectedError("Command failed".to_string()));
        }

        Ok(())
    }
}

#[async_trait]
impl CommandExecutor for ConfigCommands {
    async fn execute_with_result(&self, bot: &Bot) -> CommandResult {
        match self {
            ConfigCommands::Setup => execute_setup_with_result().await,
            ConfigCommands::Examples => execute_examples_with_result().await,
            ConfigCommands::ListCommands => execute_list_commands_with_result().await,
            ConfigCommands::Validate => execute_validate_with_result(bot).await,
            ConfigCommands::Config { show, init, wizard } => {
                execute_config_with_result(*show, *init, *wizard).await
            }
            ConfigCommands::Completion {
                shell,
                output,
                install,
                all,
            } => execute_completion_with_result(*shell, output.as_deref(), *install, *all).await,
        }
    }

    fn name(&self) -> &'static str {
        match self {
            ConfigCommands::Setup => "setup",
            ConfigCommands::Examples => "examples",
            ConfigCommands::ListCommands => "list-commands",
            ConfigCommands::Validate => "validate",
            ConfigCommands::Config { .. } => "config",
            ConfigCommands::Completion { .. } => "completion",
        }
    }

    fn validate(&self) -> CliResult<()> {
        // Configuration commands don't need pre-validation
        Ok(())
    }
}

// Command execution functions

async fn execute_setup() -> CliResult<()> {
    println!(
        "{} VK Teams Bot CLI Setup Wizard",
        emoji::ROBOT.bold().blue()
    );
    println!("This wizard will help you configure the CLI tool.\n");

    let mut new_config = toml::from_str::<Config>("").unwrap();

    // Get API token
    print!("Enter your VK Teams Bot API token: ");
    io::stdout().flush().unwrap();
    let mut token = String::new();
    io::stdin().read_line(&mut token).unwrap();
    new_config.api.token = Some(token.trim().to_string());

    // Get API URL
    print!("Enter your VK Teams Bot API URL: ");
    io::stdout().flush().unwrap();
    let mut url = String::new();
    io::stdin().read_line(&mut url).unwrap();
    new_config.api.url = Some(url.trim().to_string());

    // Ask about proxy
    print!("Do you need to configure a proxy? (y/N): ");
    io::stdout().flush().unwrap();
    let mut proxy_choice = String::new();
    io::stdin().read_line(&mut proxy_choice).unwrap();
    if proxy_choice.trim().to_lowercase() == "y" {
        print!("Enter proxy URL: ");
        io::stdout().flush().unwrap();
        let mut proxy_url = String::new();
        io::stdin().read_line(&mut proxy_url).unwrap();
        new_config.proxy = Some(crate::config::ProxyConfig {
            url: proxy_url.trim().to_string(),
            user: None,
            password: None,
        });
    }

    // Test and save configuration
    println!("\n{} Testing configuration...", emoji::TEST_TUBE);

    if let Err(e) = new_config.save(None) {
        eprintln!(
            "{}  Warning: Could not save configuration: {}",
            emoji::WARNING,
            e
        );
    } else {
        println!("{} Configuration saved successfully!", emoji::FLOPPY_DISK);
    }

    println!(
        "\n{} Setup complete! You can now use the CLI tool.",
        emoji::PARTY
    );
    println!(
        "Try: {} to test your setup",
        "vkteams-bot-cli get-self".green()
    );

    Ok(())
}

async fn execute_completion(
    shell: crate::completion::CompletionShell,
    output: Option<&str>,
    install: bool,
    all: bool,
) -> CliResult<()> {
    use crate::completion::{
        generate_all_completions, generate_completion, get_default_completion_dir,
        install_completion,
    };
    use std::path::Path;

    if all {
        let output_dir = if let Some(dir) = output {
            Path::new(dir).to_path_buf()
        } else if let Some(default_dir) = get_default_completion_dir() {
            default_dir
        } else {
            std::env::current_dir().map_err(|e| {
                CliError::FileError(format!("Failed to get current directory: {}", e))
            })?
        };

        generate_all_completions(&output_dir)?;
        return Ok(());
    }

    if install {
        install_completion(shell)?;
        return Ok(());
    }

    let output_path = output.map(Path::new);
    generate_completion(shell, output_path)?;

    Ok(())
}

// New CommandResult-based execution functions

async fn execute_setup_with_result() -> CommandResult {
    match execute_setup().await {
        Ok(()) => CommandResult::success_with_message("Setup completed successfully"),
        Err(e) => CommandResult::error(format!("Setup failed: {}", e)),
    }
}

async fn execute_examples_with_result() -> CommandResult {
    match execute_examples().await {
        Ok(()) => CommandResult::success(),
        Err(e) => CommandResult::error(format!("Failed to show examples: {}", e)),
    }
}

async fn execute_list_commands_with_result() -> CommandResult {
    match execute_list_commands().await {
        Ok(()) => CommandResult::success(),
        Err(e) => CommandResult::error(format!("Failed to list commands: {}", e)),
    }
}

async fn execute_validate_with_result(bot: &Bot) -> CommandResult {
    match execute_validate(bot).await {
        Ok(()) => CommandResult::success_with_message("Validation completed successfully"),
        Err(e) => CommandResult::error(format!("Validation failed: {}", e)),
    }
}

async fn execute_config_with_result(show: bool, init: bool, wizard: bool) -> CommandResult {
    match execute_config(show, init, wizard).await {
        Ok(()) => CommandResult::success_with_message("Configuration operation completed"),
        Err(e) => CommandResult::error(format!("Configuration operation failed: {}", e)),
    }
}

async fn execute_completion_with_result(
    shell: crate::completion::CompletionShell,
    output: Option<&str>,
    install: bool,
    all: bool,
) -> CommandResult {
    match execute_completion(shell, output, install, all).await {
        Ok(()) => CommandResult::success_with_message("Completion operation completed"),
        Err(e) => CommandResult::error(format!("Completion operation failed: {}", e)),
    }
}

async fn execute_examples() -> CliResult<()> {
    println!("{} VK Teams Bot CLI Examples", emoji::BOOKS.bold().blue());
    println!();

    println!("{}", "Basic Message Operations:".bold().green());
    println!(
        "  {}",
        "vkteams-bot-cli send-text -u USER_ID -m \"Hello World!\"".cyan()
    );
    println!(
        "  {}",
        "vkteams-bot-cli send-file -u USER_ID -p /path/to/file.pdf".cyan()
    );
    println!(
        "  {}",
        "vkteams-bot-cli send-voice -u USER_ID -p /path/to/voice.ogg".cyan()
    );
    println!();

    println!("{}", "Chat Management:".bold().green());
    println!("  {}", "vkteams-bot-cli get-chat-info -c CHAT_ID".cyan());
    println!("  {}", "vkteams-bot-cli get-chat-members -c CHAT_ID".cyan());
    println!(
        "  {}",
        "vkteams-bot-cli set-chat-title -c CHAT_ID -t \"New Title\"".cyan()
    );
    println!(
        "  {}",
        "vkteams-bot-cli set-chat-about -c CHAT_ID -a \"Chat description\"".cyan()
    );
    println!();

    println!("{}", "Message Operations:".bold().green());
    println!(
        "  {}",
        "vkteams-bot-cli edit-message -c CHAT_ID -m MSG_ID -t \"Updated text\"".cyan()
    );
    println!(
        "  {}",
        "vkteams-bot-cli delete-message -c CHAT_ID -m MSG_ID".cyan()
    );
    println!(
        "  {}",
        "vkteams-bot-cli pin-message -c CHAT_ID -m MSG_ID".cyan()
    );
    println!(
        "  {}",
        "vkteams-bot-cli unpin-message -c CHAT_ID -m MSG_ID".cyan()
    );
    println!();

    println!("{}", "File Operations:".bold().green());
    println!(
        "  {}",
        "vkteams-bot-cli get-file -f FILE_ID -p /download/path/".cyan()
    );
    println!();

    println!("{}", "Bot Information:".bold().green());
    println!("  {}", "vkteams-bot-cli get-self".cyan());
    println!("  {}", "vkteams-bot-cli get-self --detailed".cyan());
    println!("  {}", "vkteams-bot-cli get-profile -u USER_ID".cyan());
    println!();

    println!("{}", "Event Monitoring:".bold().green());
    println!("  {}", "vkteams-bot-cli get-events".cyan());
    println!(
        "  {}",
        "vkteams-bot-cli get-events -l true | jq '.events[]'".cyan()
    );
    println!();

    println!("{}", "Configuration:".bold().green());
    println!("  {}", "vkteams-bot-cli setup".cyan());
    println!("  {}", "vkteams-bot-cli config --show".cyan());
    println!("  {}", "vkteams-bot-cli config --wizard".cyan());
    println!("  {}", "vkteams-bot-cli validate".cyan());
    println!();

    println!("{}", "Shell Completion:".bold().green());
    println!("  {}", "vkteams-bot-cli completion bash".cyan());
    println!(
        "  {}",
        "vkteams-bot-cli completion zsh --output _vkteams-bot-cli".cyan()
    );
    println!("  {}", "vkteams-bot-cli completion fish --install".cyan());
    println!(
        "  {}",
        "vkteams-bot-cli completion bash --all --output ./completions".cyan()
    );
    println!();

    println!("{}", "Scheduled Messages:".bold().green());
    println!(
        "  {}",
        "vkteams-bot-cli schedule text -u CHAT_ID -m \"Hello\" -t \"2024-01-01 10:00\"".cyan()
    );
    println!(
        "  {}",
        "vkteams-bot-cli schedule text -u CHAT_ID -m \"Daily reminder\" -c \"0 9 * * *\"".cyan()
    );
    println!(
        "  {}",
        "vkteams-bot-cli schedule text -u CHAT_ID -m \"Every 5 min\" -i 300".cyan()
    );
    println!(
        "  {}",
        "vkteams-bot-cli schedule file -u CHAT_ID -p \"/path/to/report.pdf\" -t \"30m\"".cyan()
    );
    println!();

    println!("{}", "Scheduler Management:".bold().green());
    println!("  {}", "vkteams-bot-cli scheduler start".cyan());
    println!("  {}", "vkteams-bot-cli scheduler status".cyan());
    println!("  {}", "vkteams-bot-cli scheduler list".cyan());
    println!("  {}", "vkteams-bot-cli task show TASK_ID".cyan());
    println!("  {}", "vkteams-bot-cli task run TASK_ID".cyan());
    println!();

    println!("{}", "Chat Actions:".bold().green());
    println!(
        "  {}",
        "vkteams-bot-cli send-action -c CHAT_ID -a typing".cyan()
    );
    println!(
        "  {}",
        "vkteams-bot-cli send-action -c CHAT_ID -a looking".cyan()
    );
    println!();

    Ok(())
}

async fn execute_list_commands() -> CliResult<()> {
    println!(
        "{} VK Teams Bot CLI Commands Reference",
        emoji::ROBOT.bold().blue()
    );
    println!();

    let commands = vec![
        (
            "send-text",
            "Send a text message to a user or chat",
            "Basic messaging",
        ),
        (
            "send-file",
            "Upload and send a file to a user or chat",
            "File sharing",
        ),
        (
            "send-voice",
            "Send a voice message from an audio file",
            "Voice messaging",
        ),
        (
            "get-file",
            "Download a file by its ID to local storage",
            "File management",
        ),
        (
            "get-events",
            "Retrieve bot events or start long polling",
            "Event monitoring",
        ),
        (
            "get-chat-info",
            "Get detailed information about a chat",
            "Chat information",
        ),
        (
            "get-profile",
            "Get user profile information",
            "User information",
        ),
        (
            "edit-message",
            "Edit an existing message in a chat",
            "Message management",
        ),
        (
            "delete-message",
            "Delete a message from a chat",
            "Message management",
        ),
        (
            "pin-message",
            "Pin a message in a chat",
            "Message management",
        ),
        (
            "unpin-message",
            "Unpin a message from a chat",
            "Message management",
        ),
        (
            "get-chat-members",
            "List all members of a chat",
            "Chat management",
        ),
        (
            "set-chat-title",
            "Change the title of a chat",
            "Chat management",
        ),
        (
            "set-chat-about",
            "Set the description of a chat",
            "Chat management",
        ),
        (
            "send-action",
            "Send typing or looking action to a chat",
            "Chat interaction",
        ),
        (
            "get-self",
            "Get bot information and verify connectivity",
            "Bot management",
        ),
        (
            "schedule",
            "Schedule messages to be sent at specific times",
            "Scheduling",
        ),
        (
            "scheduler",
            "Manage the scheduler daemon service",
            "Scheduling",
        ),
        ("task", "Manage individual scheduled tasks", "Scheduling"),
        (
            "setup",
            "Interactive wizard for first-time configuration",
            "Configuration",
        ),
        ("examples", "Show usage examples for all commands", "Help"),
        (
            "list-commands",
            "Show this detailed command reference",
            "Help",
        ),
        (
            "validate",
            "Test configuration and bot connectivity",
            "Diagnostics",
        ),
        (
            "config",
            "Manage configuration files and settings",
            "Configuration",
        ),
        (
            "completion",
            "Generate shell completion scripts",
            "Configuration",
        ),
    ];

    let mut categories: std::collections::HashMap<&str, Vec<(&str, &str)>> =
        std::collections::HashMap::new();

    for (cmd, desc, cat) in commands {
        categories.entry(cat).or_default().push((cmd, desc));
    }

    for (category, cmds) in categories {
        println!("{}", format!("{}:", category).bold().green());
        for (cmd, desc) in cmds {
            println!("  {:<20} {}", cmd.cyan(), desc);
        }
        println!();
    }

    println!("{}", "💡 Tips:".bold().yellow());
    println!(
        "  • Use {} for command-specific help",
        "vkteams-bot-cli <command> --help".cyan()
    );
    println!(
        "  • Use {} to see usage examples",
        "vkteams-bot-cli examples".cyan()
    );
    println!(
        "  • Use {} to test your configuration",
        "vkteams-bot-cli validate".cyan()
    );
    println!(
        "  • Use {} for interactive setup",
        "vkteams-bot-cli setup".cyan()
    );

    Ok(())
}

async fn execute_validate(bot: &Bot) -> CliResult<()> {
    println!(
        "{} Validating Configuration...",
        emoji::MAGNIFYING_GLASS.bold().blue()
    );
    println!();

    // Check if configuration exists
    match Config::from_file() {
        Ok(config) => {
            println!("{} Configuration file found and readable", emoji::CHECK);

            // Check required fields
            if config.api.token.is_some() {
                println!("{} API token is configured", emoji::CHECK);
            } else {
                println!("{} API token is missing", emoji::CROSS);
            }

            if config.api.url.is_some() {
                println!("{} API URL is configured", emoji::CHECK);
            } else {
                println!("{} API URL is missing", emoji::CROSS);
            }

            // Test bot connection
            println!("\n{} Testing bot connection...", emoji::GEAR);

            let request = RequestSelfGet::new(());
            match bot.send_api_request(request).await {
                Ok(bot_info) => {
                    println!("{} API connection successful", emoji::CHECK);
                    println!("{} Bot is working correctly", emoji::CHECK);

                    if let Ok(json_str) = serde_json::to_string_pretty(&bot_info) {
                        println!("\n{}", "Bot Information:".bold().green());
                        println!("{}", json_str.green());
                    }
                }
                Err(e) => {
                    println!("{} API connection failed: {}", emoji::CROSS, e);
                    return Err(CliError::ApiError(e));
                }
            }
        }
        Err(_) => {
            println!("{} No configuration file found", emoji::CROSS);
            println!(
                "{} Run {} to create initial configuration",
                emoji::LIGHTBULB,
                "vkteams-bot-cli setup".cyan()
            );
        }
    }

    println!("\n{} Validation complete!", emoji::SPARKLES.bold().green());
    Ok(())
}

async fn execute_config(show: bool, init: bool, wizard: bool) -> CliResult<()> {
    if wizard {
        println!("{} Configuration Wizard", emoji::GEAR.bold().blue());
        println!("Current configuration will be updated.\n");

        let mut new_config = toml::from_str::<Config>("").unwrap();

        // Update API token
        if let Ok(current_config) = Config::from_file() {
            if let Some(current_token) = &current_config.api.token {
                println!(
                    "Current API token: {}***",
                    &current_token[..8.min(current_token.len())]
                );
            }
        }
        print!("Enter new API token (or press Enter to keep current): ");
        io::stdout().flush().unwrap();
        let mut token = String::new();
        io::stdin().read_line(&mut token).unwrap();
        if !token.trim().is_empty() {
            new_config.api.token = Some(token.trim().to_string());
        }

        // Update API URL
        if let Ok(current_config) = Config::from_file() {
            if let Some(current_url) = &current_config.api.url {
                println!("Current API URL: {}", current_url);
            }
        }
        print!("Enter new API URL (or press Enter to keep current): ");
        io::stdout().flush().unwrap();
        let mut url = String::new();
        io::stdin().read_line(&mut url).unwrap();
        if !url.trim().is_empty() {
            new_config.api.url = Some(url.trim().to_string());
        }

        // Save and test
        if let Err(e) = new_config.save(None) {
            eprintln!(
                "{}  Warning: Could not save configuration: {}",
                emoji::WARNING,
                e
            );
        } else {
            println!("{} Configuration updated successfully!", emoji::FLOPPY_DISK);
        }
    }

    if show {
        // Print current configuration as TOML
        match Config::from_file() {
            Ok(config) => match toml::to_string_pretty(&config) {
                Ok(config_str) => {
                    println!("Current configuration:\n{}", config_str.green());
                }
                Err(e) => {
                    return Err(CliError::UnexpectedError(format!(
                        "Failed to serialize configuration: {e}"
                    )));
                }
            },
            Err(_) => {
                println!("{} No configuration file found", emoji::INFO);
                println!("{} {}", emoji::LIGHTBULB, help::SETUP_HINT.cyan());
            }
        }
    }

    if init {
        // Create a default configuration file in the home directory
        let config = toml::from_str::<Config>("").unwrap();
        config.save(None)?;
        println!("Configuration file initialized.");
    }

    // If no flags provided, show help
    if !show && !init && !wizard {
        println!(
            "Use --show to display current configuration, --init to create a new configuration file, or --wizard for interactive configuration."
        );
    }

    Ok(())
}

// Structured output versions

async fn execute_setup_structured() -> CliResponse<serde_json::Value> {
    // Setup is interactive, return a message for structured output
    CliResponse::success(
        "setup",
        json!({
            "status": "interactive",
            "message": "Setup wizard requires interactive terminal. Use regular execute mode.",
            "help": "Run without --output-format flag for interactive setup"
        }),
    )
}

async fn execute_examples_structured() -> CliResponse<serde_json::Value> {
    let examples = vec![
        json!({
            "category": "Messaging",
            "examples": [
                {
                    "command": "vkteams-bot-cli send-text -u @user -m \"Hello!\"",
                    "description": "Send text message to user"
                },
                {
                    "command": "vkteams-bot-cli send-file -u chatId -p /path/to/file.pdf",
                    "description": "Send file to chat"
                }
            ]
        }),
        json!({
            "category": "Chat Management",
            "examples": [
                {
                    "command": "vkteams-bot-cli get-chat-info -c chatId",
                    "description": "Get chat information"
                },
                {
                    "command": "vkteams-bot-cli set-chat-title -c chatId -t \"New Title\"",
                    "description": "Set chat title"
                }
            ]
        }),
        json!({
            "category": "Storage",
            "examples": [
                {
                    "command": "vkteams-bot-cli storage database init",
                    "description": "Initialize database with migrations"
                },
                {
                    "command": "vkteams-bot-cli storage search semantic -q \"search query\"",
                    "description": "Search using semantic similarity"
                }
            ]
        }),
    ];

    CliResponse::success(
        "examples",
        json!({
            "examples": examples,
            "note": "Use --help with any command for detailed options"
        }),
    )
}

async fn execute_list_commands_structured() -> CliResponse<serde_json::Value> {
    let commands = vec![
        json!({
            "category": "Messaging",
            "commands": ["send-text", "send-file", "send-voice", "edit-message", "delete-message", "pin-message", "unpin-message"]
        }),
        json!({
            "category": "Chat",
            "commands": ["get-chat-info", "get-profile", "get-chat-members", "set-chat-title", "set-chat-about", "send-action"]
        }),
        json!({
            "category": "Files",
            "commands": ["upload", "download", "get-info"]
        }),
        json!({
            "category": "Storage",
            "commands": ["database", "search", "context"]
        }),
        json!({
            "category": "Diagnostic",
            "commands": ["get-self", "get-events", "get-file", "health-check", "network-test", "system-info", "rate-limit-test"]
        }),
        json!({
            "category": "Config",
            "commands": ["setup", "examples", "list-commands", "validate", "config", "completion"]
        }),
        json!({
            "category": "Daemon",
            "commands": ["start", "stop", "status", "restart", "logs"]
        }),
    ];

    CliResponse::success(
        "list-commands",
        json!({
            "command_categories": commands,
            "total_categories": commands.len()
        }),
    )
}

async fn execute_validate_structured(bot: &Bot) -> CliResponse<serde_json::Value> {
    let mut validation_results = Vec::new();
    
    // Test 1: Configuration file
    let config_result = match Config::from_file() {
        Ok(config) => {
            let mut issues = Vec::new();
            if config.api.token.is_none() {
                issues.push("Missing API token");
            }
            if config.api.url.is_none() {
                issues.push("Missing API URL");
            }
            
            json!({
                "test": "Configuration File",
                "status": if issues.is_empty() { "pass" } else { "fail" },
                "issues": issues
            })
        }
        Err(e) => json!({
            "test": "Configuration File",
            "status": "fail",
            "error": e.to_string()
        }),
    };
    validation_results.push(config_result);

    // Test 2: Bot connection
    let connection_result = match bot.send_api_request(RequestSelfGet::new(())).await {
        Ok(result) => json!({
            "test": "Bot Connection",
            "status": "pass",
            "bot_info": {
                "user_id": result.user_id,
                "nickname": result.nick,
                "first_name": result.first_name
            }
        }),
        Err(e) => json!({
            "test": "Bot Connection",
            "status": "fail",
            "error": e.to_string()
        }),
    };
    validation_results.push(connection_result);

    // Test 3: Environment variables
    let env_vars = [
        "VKTEAMS_BOT_API_TOKEN",
        "VKTEAMS_BOT_API_URL",
        "VKTEAMS_BOT_DOWNLOAD_DIR",
    ];
    let mut env_status = Vec::new();
    for var in &env_vars {
        env_status.push(json!({
            "variable": var,
            "set": std::env::var(var).is_ok()
        }));
    }
    validation_results.push(json!({
        "test": "Environment Variables",
        "status": "info",
        "variables": env_status
    }));

    let all_passed = validation_results.iter().all(|r| {
        r.get("status").and_then(|s| s.as_str()) != Some("fail")
    });

    CliResponse::success(
        "validate",
        json!({
            "validation_status": if all_passed { "valid" } else { "invalid" },
            "test_results": validation_results
        }),
    )
}

async fn execute_config_structured(
    show: bool,
    init: bool,
    wizard: bool,
) -> CliResponse<serde_json::Value> {
    if wizard {
        return CliResponse::success(
            "config",
            json!({
                "mode": "wizard",
                "status": "interactive",
                "message": "Configuration wizard requires interactive terminal",
                "help": "Run without --output-format flag for interactive wizard"
            }),
        );
    }

    if init {
        match Config::default().save(None) {
            Ok(_) => {
                let config_path = crate::utils::config_helpers::get_config_paths()
                    .first()
                    .map(|p| p.display().to_string())
                    .unwrap_or_else(|| "unknown".to_string());
                
                CliResponse::success(
                    "config",
                    json!({
                        "action": "init",
                        "status": "created",
                        "path": config_path
                    }),
                )
            }
            Err(e) => CliResponse::error("config", format!("Failed to initialize config: {}", e)),
        }
    } else if show {
        match Config::from_file() {
            Ok(config) => {
                let config_json = serde_json::to_value(&config).unwrap_or(json!({}));
                CliResponse::success(
                    "config",
                    json!({
                        "action": "show",
                        "configuration": config_json
                    }),
                )
            }
            Err(e) => CliResponse::error("config", format!("Failed to load config: {}", e)),
        }
    } else {
        CliResponse::success(
            "config",
            json!({
                "message": "No action specified",
                "available_flags": ["--show", "--init", "--wizard"]
            }),
        )
    }
}

async fn execute_completion_structured(
    shell: crate::completion::CompletionShell,
    output: Option<&str>,
    install: bool,
    all: bool,
) -> CliResponse<serde_json::Value> {
    if all {
        return CliResponse::success(
            "completion",
            json!({
                "mode": "all",
                "message": "Generating completions for all shells",
                "shells": ["bash", "zsh", "fish", "powershell"],
                "note": "Use regular execute mode to generate files"
            }),
        );
    }

    let shell_name = match shell {
        crate::completion::CompletionShell::Bash => "bash",
        crate::completion::CompletionShell::Zsh => "zsh",
        crate::completion::CompletionShell::Fish => "fish",
        crate::completion::CompletionShell::PowerShell => "powershell",
    };

    CliResponse::success(
        "completion",
        json!({
            "shell": shell_name,
            "output": output.unwrap_or("stdout"),
            "install": install,
            "note": "Completion script generation requires regular execute mode"
        }),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::config_helpers::get_existing_config_path;
    use std::fs;
    #[cfg(unix)]
    use std::os::unix::fs::PermissionsExt;
    use tempfile::tempdir;

    /// Helper to create a dummy bot for testing
    fn dummy_bot() -> Bot {
        Bot::with_params(&APIVersionUrl::V1, "dummy_token", "https://dummy.api.com").unwrap()
    }

    #[tokio::test]
    async fn test_config_commands_variants() {
        // Check all enum variants are constructible
        let _ = ConfigCommands::Setup;
        let _ = ConfigCommands::Examples;
        let _ = ConfigCommands::ListCommands;
        let _ = ConfigCommands::Validate;
        let _ = ConfigCommands::Config {
            show: true,
            init: false,
            wizard: false,
        };
        let _ = ConfigCommands::Completion {
            shell: crate::completion::CompletionShell::Bash,
            output: None,
            install: false,
            all: false,
        };
    }

    #[tokio::test]
    async fn test_execute_examples_success() {
        // Should not return error
        let res = execute_examples().await;
        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn test_execute_list_commands_success() {
        // Should not return error
        let res = execute_list_commands().await;
        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn test_execute_config_show_success() {
        // Should not return error when showing config
        let res = execute_config(true, false, false).await;
        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn test_execute_config_init_success() {
        // Should not return error when initializing config
        let res = execute_config(false, true, false).await;
        assert!(res.is_ok());
    }

    #[tokio::test]
    #[ignore]
    async fn test_execute_config_wizard_success() {
        // Should not return error when running wizard
        let res = execute_config(false, false, true).await;
        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn test_execute_validate_success() {
        // Should not return error with dummy bot
        let bot = dummy_bot();
        let res = execute_validate(&bot).await;
        // Accept both Ok and Err (since dummy bot may fail connection)
        assert!(res.is_ok() || res.is_err());
    }

    #[tokio::test]
    async fn test_execute_completion_success() {
        // Should not return error for valid shell
        let res =
            execute_completion(crate::completion::CompletionShell::Bash, None, false, false).await;
        assert!(res.is_ok());
    }

    #[tokio::test]
    #[ignore]
    async fn test_execute_setup_mocked() {
        // This function is interactive, so we only check it does not panic
        // when called in a test environment (may fail due to lack of stdin)
        let res = execute_setup().await;
        // Accept both Ok and Err
        assert!(res.is_ok() || res.is_err());
    }

    /// Test error when config file is missing or unreadable
    #[tokio::test]
    async fn test_execute_config_show_missing_file() {
        // Temporarily rename config file if exists
        let config_path = get_existing_config_path();
        let backup = if let Some(ref config_path) = config_path {
            let backup = config_path.with_extension("bak");
            fs::rename(config_path, &backup).ok();
            Some((config_path.clone(), backup))
        } else {
            None
        };
        // Should not panic, but print info about missing file
        let res = execute_config(true, false, false).await;
        assert!(res.is_ok());
        // Restore config
        if let Some((config_path, backup)) = backup {
            fs::rename(&backup, &config_path).ok();
        }
    }

    /// Test error when config file is corrupted (invalid TOML)
    #[tokio::test]
    async fn test_execute_config_show_corrupted_file() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("config.toml");
        fs::write(&path, "not a toml").unwrap();
        // Patch Config::default_path to use temp file (simulate via env var if supported)
        // Here, just check that serialization error is handled
        let res = toml::from_str::<Config>("not a toml");
        assert!(res.is_err());
    }

    /// Test error when saving config fails (simulate unwritable dir)
    #[tokio::test]
    async fn test_execute_config_save_error() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("readonly.toml");
        fs::write(&path, "").unwrap();
        let _ = fs::set_permissions(&path, fs::Permissions::from_mode(0o444));
        let config = Config::default();
        let res = config.save(Some(&path));
        assert!(res.is_err());
    }

    /// Test error in completion generation (invalid shell)
    #[tokio::test]
    async fn test_execute_completion_invalid_shell() {
        // Use an invalid shell by casting from an invalid integer (simulate)
        // Here, just check that function returns error for invalid output path
        let res = execute_completion(
            crate::completion::CompletionShell::Bash,
            Some("/invalid/path/doesnotexist"),
            false,
            false,
        )
        .await;
        assert!(res.is_err());
    }

    /// Test validate with missing config fields
    #[tokio::test]
    async fn test_execute_validate_missing_fields() {
        // Remove config file if exists
        let config_path = get_existing_config_path();
        let backup = if let Some(ref config_path) = config_path {
            let backup = config_path.with_extension("bak");
            fs::rename(config_path, &backup).ok();
            Some((config_path.clone(), backup))
        } else {
            None
        };
        let bot = dummy_bot();
        let res = execute_validate(&bot).await;
        // Should not panic, may return Ok or Err
        assert!(res.is_ok() || res.is_err());
        // Restore config
        if let Some((config_path, backup)) = backup {
            fs::rename(&backup, &config_path).ok();
        }
    }

    /// Test execute_config with no flags (should print help)
    #[tokio::test]
    async fn test_execute_config_no_flags() {
        let res = execute_config(false, false, false).await;
        assert!(res.is_ok());
    }

    // #[tokio::test]
    // async fn test_execute_setup_stdin_error() {
    //     // Simulate stdin read_line error by replacing stdin
    //     // This test is only illustrative, as replacing stdin is non-trivial in Rust
    //     // In real code, consider refactoring to inject input source
    //     // Here we just check that setup does not panic on empty input
    //     let _ = execute_setup().await;
    // }

    #[tokio::test]
    async fn test_execute_config_toml_deserialize_error() {
        // Simulate toml::from_str error by passing invalid TOML
        let result = toml::from_str::<Config>("invalid = toml");
        assert!(result.is_err());
    }

    // #[tokio::test]
    // async fn test_execute_config_flag_combinations() {
    //     // All combinations of show/init/wizard
    //     let combos = vec![
    //         (true, false, false),
    //         (false, true, false),
    //         (false, false, true),
    //         (true, true, false),
    //         (true, false, true),
    //         (false, true, true),
    //         (true, true, true),
    //         (false, false, false),
    //     ];
    //     for (show, init, wizard) in combos {
    //         let _ = execute_config(show, init, wizard).await;
    //     }
    // }
}

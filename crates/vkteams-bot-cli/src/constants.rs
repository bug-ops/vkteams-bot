//! Constants used throughout the CLI application
//!
//! This module centralizes all string literals, configuration defaults,
//! and other constants to improve maintainability and reduce duplication.

/// Configuration file constants
pub mod config {
    pub const CONFIG_FILE_NAME: &str = "cli_config.toml";
    pub const DEFAULT_CONFIG_DIR: &str = ".config/vkteams-bot";
    pub const SCHEDULER_DATA_FILE: &str = "scheduler_tasks.json";
    pub const ENV_PREFIX: &str = "VKTEAMS_";
}

/// Default configuration values
pub mod defaults {
    pub const TIMEOUT_SECONDS: u64 = 30;
    pub const MAX_RETRIES: u32 = 3;
    pub const MAX_FILE_SIZE_BYTES: usize = 100 * 1024 * 1024; // 100MB
    pub const BUFFER_SIZE_BYTES: usize = 64 * 1024; // 64KB
    pub const LOG_LEVEL: &str = "info";
    pub const LOG_FORMAT: &str = "text";
    pub const LOG_COLORS: bool = true;
    pub const SHOW_PROGRESS: bool = true;
    pub const PROGRESS_STYLE: &str = "unicode";
    pub const PROGRESS_REFRESH_RATE_MS: u64 = 100;
}

/// UI constants for colored output and formatting
pub mod ui {
    pub mod emoji {
        pub const ROBOT: &str = "🤖";
        pub const CHECK: &str = "✅";
        pub const CROSS: &str = "❌";
        pub const WARNING: &str = "⚠️";
        pub const INFO: &str = "ℹ️";
        pub const ROCKET: &str = "🚀";
        pub const GEAR: &str = "⚙️";
        pub const FOLDER: &str = "📁";
        pub const FILE: &str = "📄";
        pub const CLOCK: &str = "⏰";
        pub const CALENDAR: &str = "📅";
        pub const CHART: &str = "📊";
        pub const BOOKS: &str = "📚";
        pub const LIGHTBULB: &str = "💡";
        pub const SPARKLES: &str = "✨";
        pub const FLOPPY_DISK: &str = "💾";
        pub const MAGNIFYING_GLASS: &str = "🔍";
        pub const TEST_TUBE: &str = "🧪";
        pub const PARTY: &str = "🎉";
        pub const UPLOAD: &str = "⬆️";
        pub const DOWNLOAD: &str = "⬇️";
        pub const STOP: &str = "⏹️";
        pub const PLAY: &str = "▶️";
        pub const NEXT: &str = "⏭️";
        pub const CLIPBOARD: &str = "📋";
        pub const ID_BADGE: &str = "🆔";
        pub const MEMO: &str = "📝";
        pub const PACKAGE: &str = "📦";
        pub const MAILBOX_EMPTY: &str = "📭";
        pub const HASH: &str = "#️⃣";
    }

    pub mod symbols {
        pub const ARROW_RIGHT: &str = "→";
        pub const BULLET: &str = "•";
        pub const DASH: &str = "─";
        pub const PIPE: &str = "│";
        pub const CORNER: &str = "└";
        pub const TEE: &str = "├";
    }

    pub mod progress {
        pub const UNICODE_CHARS: &str = "━━╾─";
        pub const ASCII_CHARS: &str = "##-";
        pub const SPINNER_CHARS: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
    }
}

/// Error message templates
pub mod errors {
    pub const FILE_NOT_FOUND: &str = "File not found";
    pub const DIR_NOT_FOUND: &str = "Directory not found";
    pub const NOT_A_FILE: &str = "Path is not a file";
    pub const NOT_A_DIR: &str = "Path is not a directory";
    pub const WRITE_ERROR: &str = "Failed to write file";
    pub const READ_ERROR: &str = "Failed to read file";
    pub const DOWNLOAD_ERROR: &str = "Failed to download file";
    pub const API_ERROR: &str = "API Error";
    pub const INPUT_ERROR: &str = "Input Error";
    pub const UNEXPECTED_ERROR: &str = "Unexpected Error";
    pub const CONFIG_ERROR: &str = "Configuration Error";
    pub const NETWORK_ERROR: &str = "Network Error";
    pub const PERMISSION_ERROR: &str = "Permission Error";
}

/// API related constants
pub mod api {
    pub const DEFAULT_USER_AGENT: &str = "vkteams-bot-cli";
    pub const CONTENT_TYPE_JSON: &str = "application/json";
    pub const CONTENT_TYPE_MULTIPART: &str = "multipart/form-data";
    
    pub mod actions {
        pub const TYPING: &str = "typing";
        pub const LOOKING: &str = "looking";
    }
    
    pub mod endpoints {
        pub const HEALTH_CHECK: &str = "/self/get";
    }
}

/// Time and date formatting constants
pub mod time {
    pub const DATETIME_FORMAT: &str = "%Y-%m-%d %H:%M:%S UTC";
    pub const DATE_FORMAT: &str = "%Y-%m-%d";
    pub const TIME_FORMAT: &str = "%H:%M:%S";
    pub const ISO_FORMAT: &str = "%Y-%m-%dT%H:%M:%SZ";
    
    pub mod relative {
        pub const MINUTE_SUFFIX: char = 'm';
        pub const HOUR_SUFFIX: char = 'h';
        pub const DAY_SUFFIX: char = 'd';
        pub const SECOND_SUFFIX: char = 's';
    }
}

/// Scheduler constants
pub mod scheduler {
    pub const CHECK_INTERVAL_SECONDS: u64 = 60;
    pub const MAX_TASK_HISTORY: usize = 100;
    pub const DEFAULT_TASK_NAME: &str = "Unnamed Task";
    
    pub mod cron {
        pub const EVERY_MINUTE: &str = "* * * * *";
        pub const EVERY_HOUR: &str = "0 * * * *";
        pub const EVERY_DAY: &str = "0 0 * * *";
        pub const EVERY_WEEK: &str = "0 0 * * 0";
        pub const EVERY_MONTH: &str = "0 0 1 * *";
    }
}

/// Help text templates
pub mod help {
    pub const SETUP_HINT: &str = "Run 'vkteams-bot-cli setup' to configure the CLI";
    pub const CONFIG_HINT: &str = "Use 'vkteams-bot-cli config --show' to view current configuration";
    pub const VALIDATE_HINT: &str = "Use 'vkteams-bot-cli validate' to test your configuration";
    pub const EXAMPLES_HINT: &str = "Use 'vkteams-bot-cli examples' to see usage examples";
    pub const HELP_HINT: &str = "Use 'vkteams-bot-cli <command> --help' for command-specific help";
    
    pub const SCHEDULER_START_HINT: &str = "Use 'vkteams-bot-cli scheduler start' to start the scheduler daemon";
    pub const SCHEDULER_LIST_HINT: &str = "Use 'vkteams-bot-cli scheduler list' to list all tasks";
    pub const TASK_HELP_HINT: &str = "Use 'vkteams-bot-cli task --help' for task management commands";
}

/// Command categories for help display
pub mod categories {
    pub const BASIC_MESSAGING: &str = "Basic messaging";
    pub const CHAT_MANAGEMENT: &str = "Chat management";
    pub const MESSAGE_MANAGEMENT: &str = "Message management";
    pub const FILE_MANAGEMENT: &str = "File management";
    pub const EVENT_MONITORING: &str = "Event monitoring";
    pub const BOT_MANAGEMENT: &str = "Bot management";
    pub const SCHEDULING: &str = "Scheduling";
    pub const CONFIGURATION: &str = "Configuration";
    pub const DIAGNOSTICS: &str = "Diagnostics";
    pub const HELP: &str = "Help";
    pub const CHAT_INTERACTION: &str = "Chat interaction";
    pub const USER_INFORMATION: &str = "User information";
}

/// Validation patterns and limits
pub mod validation {
    pub const MIN_USERNAME_LENGTH: usize = 1;
    pub const MAX_USERNAME_LENGTH: usize = 64;
    pub const MIN_MESSAGE_LENGTH: usize = 1;
    pub const MAX_MESSAGE_LENGTH: usize = 4096;
    pub const MIN_CHAT_TITLE_LENGTH: usize = 1;
    pub const MAX_CHAT_TITLE_LENGTH: usize = 255;
    pub const MIN_FILE_SIZE: usize = 1;
    pub const MAX_TASK_NAME_LENGTH: usize = 255;
    
    pub mod patterns {
        pub const CHAT_ID_PATTERN: &str = r"^[a-zA-Z0-9_@.-]+$";
        pub const FILE_ID_PATTERN: &str = r"^[a-zA-Z0-9_-]+$";
        pub const CRON_PATTERN: &str = r"^[0-9*,-/]+\s+[0-9*,-/]+\s+[0-9*,-/]+\s+[0-9*,-/]+\s+[0-9*,-/]+$";
    }
}

/// Exit codes following UNIX conventions
pub mod exit_codes {
    pub const SUCCESS: i32 = 0;
    pub const GENERAL_ERROR: i32 = 1;
    pub const USAGE_ERROR: i32 = 64;    // EX_USAGE
    pub const DATAERR: i32 = 65;        // EX_DATAERR  
    pub const NOINPUT: i32 = 66;        // EX_NOINPUT
    pub const SOFTWARE: i32 = 70;       // EX_SOFTWARE
    pub const IOERR: i32 = 74;          // EX_IOERR
    pub const TEMPFAIL: i32 = 75;       // EX_TEMPFAIL
    pub const PROTOCOL: i32 = 76;       // EX_PROTOCOL
    pub const NOPERM: i32 = 77;         // EX_NOPERM
    pub const CONFIG: i32 = 78;         // EX_CONFIG
    pub const UNAVAILABLE: i32 = 69;    // EX_UNAVAILABLE
}
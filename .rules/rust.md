
# Claude

## Project Overview

This is a Rust-based VK Teams Bot ecosystem consisting of four main crates in a workspace:

- **vkteams-bot**: Core async VK Teams Bot API client library
- **vkteams-bot-cli**: Feature-complete CLI tool for VK Teams bot operations
- **vkteams-bot-mcp**: Model Context Protocol (MCP) server for AI/LLM integration
- **vkteams-bot-macros**: Development productivity macros

## Essential Development Commands

### Build and Test

```bash
# Build all workspace crates
cargo build --release

# Run all tests
cargo nextest run

# Run tests with all features
cargo nextest run --all-features

# Format code
cargo fmt --all

# Run linter
cargo clippy --all-targets --all-features -- -D warnings

# Check code without building
cargo check --all-targets --all-features
```

### CLI-Specific Commands (from crates/vkteams-bot-cli/)

```bash
# Build CLI with completions
make build

# Run all development checks
make ci

# Install CLI locally
make install

# Run benchmarks
make bench-all

# Test completions
make test-completions
```

### Single Test Execution

```bash
# Run specific test
cargo nextest run test_name

# Run tests in specific crate
cargo nextest run -p vkteams-bot

# Run integration tests
cargo nextest run --test integration_cli
```

## Architecture Overview

### Core Library (vkteams-bot)

- **API Layer**: Modular API methods organized by VK Teams endpoints (`/chats/`, `/messages/`, `/events/`, `/files/`, `/myself/`)
- **Bot Client**: Main `Bot` struct provides async interface to VK Teams API
- **Type System**: Comprehensive type definitions with serde serialization
- **Macro System**: `bot_api_method!` macro generates request/response structs and implementations
- **Networking**: Built on `reqwest` and `tokio` for async HTTP operations

### CLI Tool (vkteams-bot-cli)

- **Command Structure**: Modular command system with subcommands for different operations
- **Configuration**: TOML-based config with environment variable support
- **Completion System**: Shell completion generation for bash, zsh, fish, powershell
- **Progress Reporting**: Built-in progress bars and status reporting
- **Validation**: Comprehensive input validation for chat IDs, files, messages, etc.

### MCP Server (vkteams-bot-mcp)

- **Protocol Implementation**: Model Context Protocol server for AI integration
- **File Operations**: Direct file upload/download capabilities
- **Message Handling**: Send text, files, and voice messages through AI assistants

## Key Configuration

### Environment Variables

- `VKTEAMS_BOT_API_TOKEN`: Bot API token (required)
- `VKTEAMS_BOT_API_URL`: API endpoint URL (required)
- `VKTEAMS_BOT_CHAT_ID`: Default chat ID for operations

### Workspace Structure

- Root workspace with `members = ["crates/*"]`
- Edition 2024 Rust with `resolver = "2"`
- Shared package metadata in `[workspace.package]`

## Testing Strategy

The project uses comprehensive testing including:

- Unit tests for each module
- Integration tests for CLI operations
- Property-based tests using `proptest`
- Benchmark tests for performance regression detection
- JSON response validation tests

## Key Patterns

### API Method Generation

The `bot_api_method!` macro automatically generates request/response types and implementations for VK Teams API endpoints. Each generated struct implements the `BotRequest` trait.

### Error Handling

Comprehensive error types using `thiserror` with specific error variants for different failure modes (network, parsing, validation, etc.).

### Configuration Management

CLI uses layered configuration: defaults → config file → environment variables → CLI arguments.

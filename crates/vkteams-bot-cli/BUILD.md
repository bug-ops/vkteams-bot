# VK Teams Bot CLI - Build Guide

Quick guide for building and setting up VK Teams Bot CLI with shell completion.

## 🚀 Quick Start

### 1. Build

```bash
# Standard build
cargo build --release
```

### 2. Install Completions

```bash
# Generate and install completion for your shell
vkteams-bot-cli completion bash --install
vkteams-bot-cli completion zsh --install
vkteams-bot-cli completion fish --install

# Or generate to file
vkteams-bot-cli completion bash > ~/.local/share/bash-completion/completions/vkteams-bot-cli
```

### 3. Done

Your shell now has Tab completion for `vkteams-bot-cli` commands.

## 📁 Generated Completions

Use the CLI to generate completions for your shell:

```bash
# Generate to stdout
vkteams-bot-cli completion bash
vkteams-bot-cli completion zsh
vkteams-bot-cli completion fish
vkteams-bot-cli completion powershell

# Generate to file
vkteams-bot-cli completion bash --output completion.bash
```

## 🔧 Make Targets

Essential targets for development:

```bash
make build                # Build the CLI
make test                 # Run all tests
make install-completions  # Install completions using CLI
make clean                # Clean build artifacts
make demo                 # Show CLI capabilities
```

Development targets:

```bash
make dev                  # Development build
make check                # Cargo check
make fmt                  # Format code
make clippy               # Run lints
make ci                   # Full CI pipeline
```

## ⚙️ Build Features

### Runtime Completion Generation

- **Enabled by default** with `completion` feature
- Generated using `vkteams-bot-cli completion` command
- Supports all shells: bash, zsh, fish, PowerShell
- Always up-to-date with current CLI structure

### Feature Flags

```bash
# Build with completion support (default)
cargo build --release --features completion

# Build without completion support
cargo build --release --no-default-features
```

## 🔍 Troubleshooting

**Completion feature not available?**

```bash
# Check if feature is enabled
cargo build --release --features completion
```

**Completion not working?**

```bash
# Test completion generation
vkteams-bot-cli completion bash

# Install for your shell
vkteams-bot-cli completion bash --install
vkteams-bot-cli completion zsh --install

# Check your shell
echo $SHELL
```

**Manual installation needed?**

```bash
# Generate and install manually
vkteams-bot-cli completion bash > ~/.local/share/bash-completion/completions/vkteams-bot-cli
source ~/.local/share/bash-completion/completions/vkteams-bot-cli
```

## 📦 Distribution

For packaging and distribution:

```bash
# Build release
cargo build --release

# Package includes:
# - target/release/vkteams-bot-cli (binary with completion generation capability)
# Users can generate completions using: vkteams-bot-cli completion <shell>
```

## 🎯 Key Benefits

- ✅ **Runtime generation**: Always matches current CLI structure
- ✅ **Simple installation**: Built-in installation command
- ✅ **Always up-to-date**: Generated from actual command definitions
- ✅ **Cross-platform**: Works on Linux, macOS, Windows
- ✅ **No build dependencies**: Cleaner build process

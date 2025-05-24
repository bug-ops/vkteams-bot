# VK Teams Bot CLI - Build Guide

Quick guide for building and setting up VK Teams Bot CLI with automatic shell completion.

## 🚀 Quick Start

### 1. Build
```bash
# Standard build (completions auto-generated)
cargo build --release

# Force completion generation if needed
VKTEAMS_GENERATE_COMPLETIONS=1 cargo build --release
```

### 2. Install Completions
```bash
# Use auto-generated installer (recommended)
./target/completions/install-completions.sh

# Or use Makefile
make install-completions
```

### 3. Done!
Your shell now has Tab completion for `vkteams-bot-cli` commands.

## 📁 What Gets Generated

After building, you'll find:
```
target/completions/
├── vkteams-bot-cli.bash          # Bash completion
├── _vkteams-bot-cli              # Zsh completion  
├── vkteams-bot-cli.fish          # Fish completion
├── vkteams-bot-cli.ps1           # PowerShell completion
└── install-completions.sh       # Auto-generated installer
```

## 🔧 Make Targets

Essential targets for development:

```bash
make build                # Build with completions
make test                 # Run all tests
make install-completions  # Install completions
make test-completions     # Test completion system
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

### Automatic Completion Generation
- **Enabled by default** with `completion` feature
- Generated during `cargo build --release`
- Stored in `target/completions/`
- Auto-generates installation script

### Build Script (`build.rs`)
- Creates completion scripts at build time
- Supports all shells: bash, zsh, fish, PowerShell
- Generates smart installer with shell auto-detection
- Falls back gracefully if generation fails

### Feature Flags
```bash
# Build with completions (default)
cargo build --release --features completion

# Build without completions
cargo build --release --no-default-features

# Force completion generation
VKTEAMS_GENERATE_COMPLETIONS=1 cargo build --release
```

## 🔍 Troubleshooting

**No completions generated?**
```bash
# Check if feature is enabled
cargo build --release --features completion

# Force generation
VKTEAMS_GENERATE_COMPLETIONS=1 cargo build --release

# Check output
ls -la target/completions/
```

**Installation script not working?**
```bash
# Make sure it's executable
chmod +x target/completions/install-completions.sh

# Check your shell
echo $SHELL

# Manual installation
cp target/completions/vkteams-bot-cli.bash ~/.bashrc
```

**CLI completion not working?**
```bash
# Test completion generation
./target/release/vkteams-bot-cli completion bash

# Use runtime installer
./target/release/vkteams-bot-cli completion bash --install
```

## 📦 Distribution

For packaging and distribution:
```bash
# Build release with completions
cargo build --release

# Package includes:
# - target/release/vkteams-bot-cli (binary)
# - target/completions/* (completion scripts)
# - target/completions/install-completions.sh (installer)
```

## 🎯 Key Benefits

- ✅ **Zero-config**: Completions generated automatically
- ✅ **One-click install**: Auto-generated installer script
- ✅ **Always up-to-date**: Generated from current code
- ✅ **Cross-platform**: Works on Linux, macOS, Windows
- ✅ **Fallback safe**: Runtime generation if build fails
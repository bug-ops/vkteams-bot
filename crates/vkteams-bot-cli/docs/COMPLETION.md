# Shell Completion for VK Teams Bot CLI

This document provides comprehensive information about shell completion support in the VK Teams Bot CLI.

## Overview

Shell completion allows you to use the Tab key to automatically complete commands, subcommands, options, and arguments. This makes the CLI faster and more convenient to use by reducing typing and helping you discover available commands.

## Supported Shells

The VK Teams Bot CLI supports completion for the following shells:

- **Bash** 4.0+ (Linux, macOS, Windows with WSL/Git Bash)
- **Zsh** 5.0+ (macOS default, Linux, Windows with WSL)
- **Fish** 3.0+ (Cross-platform)
- **PowerShell** 5.0+ / PowerShell Core 6.0+ (Windows, Linux, macOS)

## Quick Installation

### Automatic Installation (Recommended)

VK Teams Bot CLI automatically generates completion scripts during build time. The easiest way to install them:

```bash
# Use the auto-generated installation script (created during build)
target/completions/install-completions.sh

# Or install for your current shell using the CLI
vkteams-bot-cli completion bash --install
vkteams-bot-cli completion zsh --install
vkteams-bot-cli completion fish --install
vkteams-bot-cli completion powershell --install
```

### Using Prebuilt Completions

When you build VK Teams Bot CLI, completion scripts are automatically generated and stored in `target/completions/`:

```bash
# Build with completions (enabled by default)
cargo build --release

# Use the generated installation script
./target/completions/install-completions.sh

# Or manually copy the files
ls target/completions/
# vkteams-bot-cli.bash  _vkteams-bot-cli  vkteams-bot-cli.fish  vkteams-bot-cli.ps1
```

### Legacy Installation Script

For manual installation, use the provided script:

```bash
# Download and run the installation script
curl -sSL https://raw.githubusercontent.com/bug-ops/vkteams-bot/main/crates/vkteams-bot-cli/scripts/install-completion.sh | bash

# Or for a specific shell
./scripts/install-completion.sh bash
```

## Manual Installation

### Bash

#### Method 1: User-specific installation
```bash
# Create completion directory
mkdir -p ~/.local/share/bash-completion/completions

# Generate completion script
vkteams-bot-cli completion bash > ~/.local/share/bash-completion/completions/vkteams-bot-cli

# Add to your ~/.bashrc
echo 'source ~/.local/share/bash-completion/completions/vkteams-bot-cli' >> ~/.bashrc

# Reload your shell
source ~/.bashrc
```

#### Method 2: System-wide installation (requires sudo)
```bash
# Generate and install system-wide
sudo vkteams-bot-cli completion bash > /etc/bash_completion.d/vkteams-bot-cli
```

### Zsh

#### Method 1: Using local site-functions
```bash
# Create completion directory
mkdir -p ~/.local/share/zsh/site-functions

# Generate completion script
vkteams-bot-cli completion zsh > ~/.local/share/zsh/site-functions/_vkteams-bot-cli

# Add to your ~/.zshrc
echo 'fpath=(~/.local/share/zsh/site-functions $fpath)' >> ~/.zshrc
echo 'autoload -Uz compinit && compinit' >> ~/.zshrc

# Reload completions
autoload -Uz compinit && compinit
```

#### Method 2: Oh My Zsh
```bash
# Generate completion for Oh My Zsh
vkteams-bot-cli completion zsh > ~/.oh-my-zsh/completions/_vkteams-bot-cli

# Reload completions
compinit
```

#### Method 3: System-wide (macOS with Homebrew)
```bash
# Install to Homebrew's zsh completions directory
vkteams-bot-cli completion zsh > $(brew --prefix)/share/zsh/site-functions/_vkteams-bot-cli
```

### Fish

```bash
# Fish automatically loads completions from this directory
mkdir -p ~/.config/fish/completions
vkteams-bot-cli completion fish > ~/.config/fish/completions/vkteams-bot-cli.fish

# Optionally update completions cache
fish_update_completions
```

### PowerShell

#### Windows PowerShell
```powershell
# Create Scripts directory
New-Item -ItemType Directory -Force -Path "$env:USERPROFILE\Documents\WindowsPowerShell\Scripts"

# Generate completion script
vkteams-bot-cli completion powershell > "$env:USERPROFILE\Documents\WindowsPowerShell\Scripts\vkteams-bot-cli-completion.ps1"

# Add to your profile
Add-Content $PROFILE ". `"$env:USERPROFILE\Documents\WindowsPowerShell\Scripts\vkteams-bot-cli-completion.ps1`""
```

#### PowerShell Core (pwsh)
```powershell
# Create Scripts directory
New-Item -ItemType Directory -Force -Path "$env:USERPROFILE\Documents\PowerShell\Scripts"

# Generate completion script
vkteams-bot-cli completion powershell > "$env:USERPROFILE\Documents\PowerShell\Scripts\vkteams-bot-cli-completion.ps1"

# Add to your profile
Add-Content $PROFILE ". `"$env:USERPROFILE\Documents\PowerShell\Scripts\vkteams-bot-cli-completion.ps1`""
```

## Usage Examples

Once installed, you can use tab completion in various ways:

### Basic Command Completion
```bash
# Complete main commands
vkteams-bot-cli <Tab>
# Shows: send-text, send-file, send-voice, get-chat-info, schedule, etc.

# Complete subcommands
vkteams-bot-cli schedule <Tab>
# Shows: text, file, voice, action

vkteams-bot-cli scheduler <Tab>
# Shows: start, stop, status, list
```

### Option Completion
```bash
# Complete options
vkteams-bot-cli send-text --<Tab>
# Shows: --chat-id, --message, --help, --verbose, etc.

# Complete configuration options
vkteams-bot-cli config --<Tab>
# Shows: --show, --init, --wizard, --help
```

### File Path Completion
```bash
# Complete file paths
vkteams-bot-cli send-file -u user123 -p /path/to/<Tab>
# Shows available files in the directory

# Complete config file paths
vkteams-bot-cli --config ~/.config/<Tab>
# Shows available config files
```

### Value Completion
```bash
# Complete shell types for completion command
vkteams-bot-cli completion <Tab>
# Shows: bash, zsh, fish, powershell

# Complete output formats
vkteams-bot-cli --output <Tab>
# Shows: pretty, json, table, quiet
```

## Advanced Configuration

### Custom Completion Scripts

The CLI automatically uses prebuilt completions when available, but you can still generate them manually:

```bash
# Generate to stdout for piping (uses prebuilt if available)
vkteams-bot-cli completion bash | less

# Generate to custom location
vkteams-bot-cli completion zsh --output /custom/path/_vkteams-bot-cli

# Generate all completions to a directory (copies prebuilt if available)
vkteams-bot-cli completion bash --all --output ./completions/
```

### Integration with Build Systems

Completions are now automatically generated during the build process:

```bash
#!/bin/bash
# build.sh

# Build the CLI (completions generated automatically)
cargo build --release --features completion

# Completions are now available in target/completions/
ls target/completions/
# vkteams-bot-cli.bash  _vkteams-bot-cli  vkteams-bot-cli.fish  vkteams-bot-cli.ps1  install-completions.sh

# Use the generated installation script
./target/completions/install-completions.sh

echo "Completions ready for distribution!"
```

### Build-time Completion Generation

Completions are generated automatically when building with the `completion` feature (enabled by default):

```bash
# Standard build with completions
cargo build --release

# Force completion generation
VKTEAMS_GENERATE_COMPLETIONS=1 cargo build --release

# Build without completions
cargo build --release --no-default-features
```

### Docker Container Usage

For containerized environments, use the prebuilt completion scripts:

```dockerfile
# Dockerfile
FROM ubuntu:latest

# Install the CLI
COPY target/release/vkteams-bot-cli /usr/local/bin/

# Copy prebuilt completions (generated during host build)
COPY target/completions/ /usr/local/share/vkteams-bot-cli/completions/

# Install completions during container build
RUN /usr/local/share/vkteams-bot-cli/completions/install-completions.sh bash

# Or install manually
RUN mkdir -p /etc/bash_completion.d && \
    cp /usr/local/share/vkteams-bot-cli/completions/vkteams-bot-cli.bash /etc/bash_completion.d/vkteams-bot-cli
```

## Troubleshooting

### Completion Not Working

1. **Verify installation:**
   ```bash
   # Check if completion file exists
   ls -la ~/.local/share/bash-completion/completions/vkteams-bot-cli
   
   # Test completion generation
   vkteams-bot-cli completion bash > /tmp/test-completion.bash
   ```

2. **Check shell configuration:**
   ```bash
   # Bash: ensure bash-completion is enabled
   grep -i completion ~/.bashrc ~/.bash_profile
   
   # Zsh: check if compinit is called
   grep -i compinit ~/.zshrc
   ```

3. **Reload shell configuration:**
   ```bash
   # Bash
   source ~/.bashrc
   
   # Zsh
   autoload -Uz compinit && compinit
   
   # Fish
   fish_update_completions
   ```

### Completion Performance Issues

If completion is slow, try these optimizations:

1. **Reduce completion scope:**
   ```bash
   # Generate minimal completion
   vkteams-bot-cli completion bash --minimal > ~/.local/share/bash-completion/completions/vkteams-bot-cli
   ```

2. **Cache completion results:**
   ```bash
   # Use completion caching in zsh
   echo 'zstyle ":completion:*" use-cache yes' >> ~/.zshrc
   echo 'zstyle ":completion:*" cache-path ~/.zsh/cache' >> ~/.zshrc
   ```

### Shell-Specific Issues

#### Bash
- Ensure `bash-completion` package is installed
- Check that `/etc/bash_completion` is sourced in your shell

#### Zsh
- Make sure `autoload -Uz compinit && compinit` is in your `.zshrc`
- Try rebuilding completion cache: `rm ~/.zcompdump && compinit`

#### Fish
- Run `fish_update_completions` after installing
- Check Fish version: `fish --version` (requires 3.0+)

#### PowerShell
- Verify execution policy allows script execution: `Get-ExecutionPolicy`
- Set appropriate policy if needed: `Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser`

### Common Error Messages

**"Command not found: vkteams-bot-cli"**
- CLI is not installed or not in PATH
- Install with: `cargo install vkteams-bot-cli`

**"Permission denied"**
- Check file permissions: `chmod +x completion-script`
- For system-wide installation, use `sudo`

**"Completion not showing"**
- Restart terminal or source configuration
- Check if completion system is initialized

## Development

### Testing Completions

When developing or modifying completions:

```bash
# Test prebuilt completion (if available)
vkteams-bot-cli completion bash > /tmp/test.bash
source /tmp/test.bash

# Test build-time generation
VKTEAMS_GENERATE_COMPLETIONS=1 cargo build --release
ls target/completions/

# Test specific completions
complete -p vkteams-bot-cli  # Show current completion settings

# Use Makefile for comprehensive testing
make test-completions
```

### Debugging

Enable debug output for completion issues:

```bash
# Bash debug
set -x
vkteams-bot-cli <Tab><Tab>
set +x

# Zsh debug
zstyle ':completion:*' verbose yes
```

### Contributing

To improve completion support:

1. Test with different shell versions
2. Add value hints for new argument types in command definitions
3. Update completion logic in `src/completion.rs`
4. Update build script in `build.rs` for new commands
5. Test build-time generation: `VKTEAMS_GENERATE_COMPLETIONS=1 cargo build`
6. Test installation scripts on different platforms
7. Verify prebuilt completions work correctly

## Platform-Specific Notes

### macOS
- Default shell is zsh (macOS Catalina+)
- May need to install bash-completion via Homebrew
- System Integrity Protection may prevent system-wide installation

### Linux
- Most distributions include bash-completion
- Package managers may provide completion packages
- Check distribution-specific completion directories

### Windows
- PowerShell is recommended over Command Prompt
- Git Bash supports bash completion
- Windows Subsystem for Linux (WSL) supports all shell types

### FreeBSD/Other Unix
- May require manual installation of completion frameworks
- Check shell and completion system documentation

## See Also

- [Main CLI Documentation](../README.md)
- [Configuration Guide](../README.md#configuration)
- [Examples](../README.md#examples)
- [Clap Completion Documentation](https://docs.rs/clap_complete/)
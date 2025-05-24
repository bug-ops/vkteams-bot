# VK Teams Bot CLI Makefile
# Simplified build automation focused on essential tasks

# Project configuration
PROJECT_NAME = vkteams-bot-cli
BINARY_NAME = vkteams-bot-cli

# Colors for output
COLOR_RESET = \033[0m
COLOR_BOLD = \033[1m
COLOR_GREEN = \033[32m
COLOR_BLUE = \033[34m
COLOR_YELLOW = \033[33m

# Default target
.PHONY: all
all: build

# Help target
.PHONY: help
help:
	@echo "$(COLOR_BOLD)VK Teams Bot CLI Build System$(COLOR_RESET)"
	@echo ""
	@echo "$(COLOR_GREEN)Build Targets:$(COLOR_RESET)"
	@echo "  build                 Build the CLI binary with completions"
	@echo "  release               Build optimized release binary"
	@echo "  clean                 Clean build artifacts"
	@echo ""
	@echo "$(COLOR_GREEN)Development Targets:$(COLOR_RESET)"
	@echo "  test                  Run all tests"
	@echo "  check                 Run cargo check"
	@echo "  fmt                   Format code with rustfmt"
	@echo "  clippy                Run clippy lints"
	@echo ""
	@echo "$(COLOR_GREEN)Completion Targets:$(COLOR_RESET)"
	@echo "  install-completions   Install completions using auto-generated script"
	@echo "  test-completions      Test completion generation"
	@echo ""
	@echo "$(COLOR_GREEN)Installation Targets:$(COLOR_RESET)"
	@echo "  install               Install binary and completions"
	@echo ""
	@echo "$(COLOR_GREEN)Utility Targets:$(COLOR_RESET)"
	@echo "  demo                  Show CLI capabilities"

# Build targets
.PHONY: build
build:
	@echo "$(COLOR_BLUE)Building $(PROJECT_NAME) with completions...$(COLOR_RESET)"
	cargo build --release --features completion
	@if [ -d "target/completions" ]; then \
		echo "$(COLOR_GREEN)✓ Completions auto-generated in target/completions/$(COLOR_RESET)"; \
	fi

.PHONY: release
release: build

.PHONY: clean
clean:
	@echo "$(COLOR_BLUE)Cleaning build artifacts...$(COLOR_RESET)"
	cargo clean
	rm -rf target/completions

# Development targets
.PHONY: test
test:
	@echo "$(COLOR_BLUE)Running tests...$(COLOR_RESET)"
	cargo test --all-features

.PHONY: check
check:
	@echo "$(COLOR_BLUE)Running cargo check...$(COLOR_RESET)"
	cargo check --all-targets --all-features

.PHONY: fmt
fmt:
	@echo "$(COLOR_BLUE)Formatting code...$(COLOR_RESET)"
	cargo fmt --all

.PHONY: clippy
clippy:
	@echo "$(COLOR_BLUE)Running clippy...$(COLOR_RESET)"
	cargo clippy --all-targets --all-features -- -D warnings

# Completion targets
.PHONY: install-completions
install-completions: build
	@echo "$(COLOR_BLUE)Installing completions...$(COLOR_RESET)"
	@if [ -f "target/completions/install-completions.sh" ]; then \
		bash target/completions/install-completions.sh; \
	else \
		echo "$(COLOR_YELLOW)No auto-generated installer found. Run 'make build' first.$(COLOR_RESET)"; \
	fi

.PHONY: test-completions
test-completions: build
	@echo "$(COLOR_BLUE)Testing completion generation...$(COLOR_RESET)"
	@if [ -d "target/completions" ]; then \
		echo "$(COLOR_GREEN)✓ Build script generated completions:$(COLOR_RESET)"; \
		ls -la target/completions/; \
		echo "$(COLOR_GREEN)✓ Testing CLI completion commands:$(COLOR_RESET)"; \
		./target/release/$(BINARY_NAME) completion bash > /tmp/test-completion-bash && echo "✓ Bash completion works"; \
		./target/release/$(BINARY_NAME) completion zsh > /tmp/test-completion-zsh && echo "✓ Zsh completion works"; \
	else \
		echo "$(COLOR_YELLOW)No completions found. Build may have failed.$(COLOR_RESET)"; \
		exit 1; \
	fi

# Installation targets
.PHONY: install
install: build
	@echo "$(COLOR_BLUE)Installing $(PROJECT_NAME)...$(COLOR_RESET)"
	@if command -v cargo >/dev/null 2>&1; then \
		cargo install --path . --force; \
		echo "$(COLOR_GREEN)✓ Binary installed via cargo$(COLOR_RESET)"; \
	else \
		echo "$(COLOR_YELLOW)Cargo not found. Manual installation required.$(COLOR_RESET)"; \
	fi
	@$(MAKE) install-completions

# Utility targets
.PHONY: demo
demo: build
	@echo "$(COLOR_BLUE)VK Teams Bot CLI Demo$(COLOR_RESET)"
	@echo "$(COLOR_GREEN)Available commands:$(COLOR_RESET)"
	@./target/release/$(BINARY_NAME) list-commands 2>/dev/null || echo "Use --help to see commands"
	@echo ""
	@echo "$(COLOR_GREEN)Completion files:$(COLOR_RESET)"
	@if [ -d "target/completions" ]; then \
		ls -la target/completions/; \
	else \
		echo "No completions generated yet. Run 'make build' first."; \
	fi

# Force targets for development
.PHONY: force-build
force-build: clean
	@echo "$(COLOR_BLUE)Force rebuilding with completions...$(COLOR_RESET)"
	VKTEAMS_GENERATE_COMPLETIONS=1 cargo build --release --features completion

.PHONY: dev
dev:
	@echo "$(COLOR_BLUE)Development build...$(COLOR_RESET)"
	cargo build --features completion

# CI targets
.PHONY: ci
ci: check test clippy fmt-check test-completions
	@echo "$(COLOR_GREEN)CI pipeline completed successfully!$(COLOR_RESET)"

.PHONY: fmt-check
fmt-check:
	@echo "$(COLOR_BLUE)Checking code formatting...$(COLOR_RESET)"
	cargo fmt --all -- --check
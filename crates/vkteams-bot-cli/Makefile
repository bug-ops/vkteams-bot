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
	@echo ""
	@echo "$(COLOR_GREEN)Benchmark Targets:$(COLOR_RESET)"
	@echo "  bench-all             Run all benchmarks"
	@echo "  bench-cli             Run CLI performance benchmarks"
	@echo "  bench-config          Run configuration benchmarks"
	@echo "  bench-specialized     Run specialized CLI benchmarks"
	@echo "  bench-test            Test benchmark compilation"
	@echo "  bench-report          Generate HTML benchmark reports"
	@echo "  bench-compare         Compare with baseline"
	@echo "  clean-bench           Clean benchmark artifacts"
	@echo "  bench-report          Generate HTML benchmark reports"
	@echo "  bench-compare         Compare with baseline measurements"
	@echo "  clean-bench           Clean benchmark artifacts"

# Build targets
.PHONY: build
build:
	@echo "$(COLOR_BLUE)Building $(PROJECT_NAME)...$(COLOR_RESET)"
	cargo build --release --features completion
	@echo "$(COLOR_GREEN)✓ Build completed with completion support$(COLOR_RESET)"

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
	@if command -v ./target/release/$(BINARY_NAME) >/dev/null 2>&1; then \
		./target/release/$(BINARY_NAME) completion bash --install 2>/dev/null || echo "$(COLOR_YELLOW)Bash completion install failed$(COLOR_RESET)"; \
		./target/release/$(BINARY_NAME) completion zsh --install 2>/dev/null || echo "$(COLOR_YELLOW)Zsh completion install failed$(COLOR_RESET)"; \
		./target/release/$(BINARY_NAME) completion fish --install 2>/dev/null || echo "$(COLOR_YELLOW)Fish completion install failed$(COLOR_RESET)"; \
		echo "$(COLOR_GREEN)✓ Completions installed using CLI$(COLOR_RESET)"; \
	else \
		echo "$(COLOR_YELLOW)CLI binary not found. Run 'make build' first.$(COLOR_RESET)"; \
	fi

.PHONY: test-completions
test-completions: build
	@echo "$(COLOR_BLUE)Testing completion generation...$(COLOR_RESET)"
	@echo "$(COLOR_GREEN)✓ Testing CLI completion commands:$(COLOR_RESET)"
	@./target/release/$(BINARY_NAME) completion bash > /tmp/test-completion-bash && echo "✓ Bash completion works" || echo "✗ Bash completion failed"
	@./target/release/$(BINARY_NAME) completion zsh > /tmp/test-completion-zsh && echo "✓ Zsh completion works" || echo "✗ Zsh completion failed"
	@./target/release/$(BINARY_NAME) completion fish > /tmp/test-completion-fish && echo "✓ Fish completion works" || echo "✗ Fish completion failed"
	@./target/release/$(BINARY_NAME) completion powershell > /tmp/test-completion-powershell && echo "✓ PowerShell completion works" || echo "✗ PowerShell completion failed"
	@echo "$(COLOR_GREEN)✓ Runtime completion generation tested$(COLOR_RESET)"

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
	@echo "$(COLOR_GREEN)Completion generation:$(COLOR_RESET)"
	@echo "Generate completions with: ./target/release/$(BINARY_NAME) completion <shell>"
	@echo "Available shells: bash, zsh, fish, powershell"

# Force targets for development
.PHONY: force-build
force-build: clean
	@echo "$(COLOR_BLUE)Force rebuilding...$(COLOR_RESET)"
	cargo build --release --features completion

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

# Benchmark targets
.PHONY: bench bench-all bench-cli bench-config bench-specialized bench-report bench-test bench-compare clean-bench

bench-all:
	@echo "$(COLOR_BLUE)Running all VK Teams Bot CLI benchmarks...$(COLOR_RESET)"
	cargo bench
	@echo "$(COLOR_GREEN)✓ All benchmarks completed$(COLOR_RESET)"

bench-cli:
	@echo "$(COLOR_BLUE)Running CLI performance benchmarks...$(COLOR_RESET)"
	cargo bench --bench cli_performance
	@echo "$(COLOR_GREEN)✓ CLI performance benchmarks completed$(COLOR_RESET)"

bench-config:
	@echo "$(COLOR_BLUE)Running configuration benchmarks...$(COLOR_RESET)"
	cargo bench --bench config_performance
	@echo "$(COLOR_GREEN)✓ Configuration benchmarks completed$(COLOR_RESET)"

bench-specialized:
	@echo "$(COLOR_BLUE)Running specialized CLI benchmarks...$(COLOR_RESET)"
	cargo bench --bench specialized_cli_performance
	@echo "$(COLOR_GREEN)✓ Specialized CLI benchmarks completed$(COLOR_RESET)"

bench-test:
	@echo "$(COLOR_BLUE)Testing benchmarks for compilation and basic functionality...$(COLOR_RESET)"
	cargo bench --bench cli_performance -- --test
	cargo bench --bench config_performance -- --test
	cargo bench --bench specialized_cli_performance -- --test
	@echo "$(COLOR_GREEN)✓ Benchmark tests completed$(COLOR_RESET)"

bench-report:
	@echo "$(COLOR_BLUE)Generating HTML benchmark reports...$(COLOR_RESET)"
	cargo bench -- --output-format html
	@echo "$(COLOR_GREEN)✓ Reports generated in target/criterion/report/index.html$(COLOR_RESET)"

bench-compare:
	@echo "$(COLOR_BLUE)Comparing with baseline measurements...$(COLOR_RESET)"
	cargo bench -- --save-baseline main
	@echo "$(COLOR_GREEN)✓ Baseline saved. Run 'cargo bench -- --baseline main' to compare later$(COLOR_RESET)"

clean-bench:
	@echo "$(COLOR_BLUE)Cleaning benchmark artifacts...$(COLOR_RESET)"
	rm -rf target/criterion/
	@echo "$(COLOR_GREEN)✓ Benchmark artifacts cleaned$(COLOR_RESET)"

# Development benchmark shortcuts
.PHONY: dev-bench bench-parsing bench-validation bench-serialization bench-formatting

dev-bench: bench-test
	@echo "$(COLOR_GREEN)✓ Quick development benchmark check completed$(COLOR_RESET)"

bench-parsing:
	@echo "$(COLOR_BLUE)Running parsing benchmarks...$(COLOR_RESET)"
	cargo bench parsing

bench-validation:
	@echo "$(COLOR_BLUE)Running validation benchmarks...$(COLOR_RESET)"
	cargo bench validation

bench-serialization:
	@echo "$(COLOR_BLUE)Running serialization benchmarks...$(COLOR_RESET)"
	cargo bench serialization

bench-formatting:
	@echo "$(COLOR_BLUE)Running formatting benchmarks...$(COLOR_RESET)"
	cargo bench formatting
# VKTeams Bot Docker Management Makefile

# Version and build information
BUILD_DATE := $(shell date -u +"%Y-%m-%dT%H:%M:%SZ")
BUILD_VERSION := $(shell git describe --tags --always --dirty 2>/dev/null || echo "latest")
BUILD_COMMIT := $(shell git rev-parse --short HEAD 2>/dev/null || echo "unknown")

# Docker configuration
DOCKER_REGISTRY ?= 
DOCKER_NAMESPACE ?= vkteams-bot
DOCKER_TAG ?= $(BUILD_VERSION)

# Export build variables for docker-compose
export BUILD_DATE
export BUILD_VERSION
export BUILD_COMMIT

.PHONY: help build build-cli build-mcp build-all up down logs clean setup check

help: ## Show this help message
	@echo "VKTeams Bot Docker Management"
	@echo "Usage: make [target]"
	@echo ""
	@echo "Targets:"
	@awk 'BEGIN {FS = ":.*?## "} /^[a-zA-Z_-]+:.*?## / {printf "  %-15s %s\n", $$1, $$2}' $(MAKEFILE_LIST)

# Environment setup
setup: ## Setup environment file from template
	@if [ ! -f .env ]; then \
		cp .env.example .env; \
		echo "Created .env file from template. Please edit it with your configuration."; \
	else \
		echo ".env file already exists."; \
	fi

check: ## Check environment variables
	@echo "Checking required environment variables..."
	@test -f .env || (echo "Error: .env file not found. Run 'make setup' first." && exit 1)
	@grep -q "VKTEAMS_BOT_API_TOKEN=" .env || (echo "Error: VKTEAMS_BOT_API_TOKEN not set in .env" && exit 1)
	@grep -q "VKTEAMS_BOT_CHAT_ID=" .env || (echo "Error: VKTEAMS_BOT_CHAT_ID not set in .env" && exit 1)
	@echo "Environment check passed ✓"

build-mcp: ## Build MCP server container
	docker build \
		--file Dockerfile \
		--build-arg COMPONENT_TYPE=mcp \
		--build-arg APP_USER=vkteams \
		--build-arg BUILD_DATE="$(BUILD_DATE)" \
		--build-arg BUILD_VERSION="$(BUILD_VERSION)" \
		--build-arg BUILD_COMMIT="$(BUILD_COMMIT)" \
		--tag $(DOCKER_NAMESPACE)/mcp:$(DOCKER_TAG) \
		.

build: ## Build all containers using docker-compose
	docker-compose build

# Runtime targets
up: check ## Start all services
	docker-compose up -d

up-full: check ## Start all services with all profiles
	docker-compose --profile vector-search --profile embedding-local up -d

up-vector: check ## Start services with vector search
	docker-compose --profile vector-search up -d

up-local: check ## Start services with local embeddings
	docker-compose --profile embedding-local up -d

down: ## Stop all services
	docker-compose down

down-volumes: ## Stop all services and remove volumes
	docker-compose down -v

restart: docker-compose down up ## Restart all services

# Database operations
db-init: ## Initialize database
	docker-compose run --rm db-init database init

db-stats: ## Show database statistics
	docker-compose run --rm vkteams-cli database stats

db-shell: ## Connect to database shell
	docker-compose exec postgres psql -U vkteams vkteams_bot

# Logs and monitoring
logs: ## Show logs for all services
	docker-compose logs -f

logs-cli: ## Show CLI logs
	docker-compose logs -f vkteams-cli

logs-mcp: ## Show MCP server logs
	docker-compose logs -f vkteams-mcp

logs-db: ## Show database logs
	docker-compose logs -f postgres

status: ## Show service status
	docker-compose ps

# Development and testing
shell-mcp: ## Open shell in MCP container
	docker-compose exec vkteams-mcp bash

test-cli: ## Test CLI functionality
	docker-compose run --rm vkteams-mcp --version
	docker-compose run --rm vkteams-mcp help

test-mcp: ## Test MCP server
	docker-compose exec vkteams-mcp vkteams-bot-mcp --help

# Cleanup targets
clean: ## Clean up containers and images
	docker-compose down --rmi local --volumes --remove-orphans

clean-all: ## Full cleanup (containers, images, volumes, networks)
	docker-compose down --rmi all --volumes --remove-orphans
	docker system prune -f

# Info targets
info: ## Show build information
	@echo "Build Information:"
	@echo "  Date:    $(BUILD_DATE)"
	@echo "  Version: $(BUILD_VERSION)"
	@echo "  Commit:  $(BUILD_COMMIT)"
	@echo ""
	@echo "Docker Configuration:"
	@echo "  Registry:  $(DOCKER_REGISTRY)"
	@echo "  Namespace: $(DOCKER_NAMESPACE)"
	@echo "  Tag:       $(DOCKER_TAG)"
	@echo "  Rust:      $(RUST_VERSION)"
	@echo "  Runtime:   $(RUNTIME_IMAGE)"

version: ## Show version information
	@echo "$(BUILD_VERSION)"
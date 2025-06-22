# Universal multi-stage Dockerfile for VK Teams Bot components
# Can build CLI, MCP server, or any other Rust binary using build arguments

ARG RUST_VERSION=1.87.0
FROM rust:${RUST_VERSION}-slim AS builder

# Build arguments
ARG FEATURES

# Install system dependencies for building
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Install Rust toolchain components
RUN rustup target add aarch64-unknown-linux-musl

# Set working directory
WORKDIR /app

# Copy Cargo files and crates
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates

# Pre-build dependencies with features if specified
RUN if [ -n "$FEATURES" ]; then \
    cargo build --release --features "$FEATURES"; \
    else \
    cargo build --release; \
    fi

# Runtime stage
FROM ubuntu:latest AS runtime
WORKDIR /app

# Runtime build arguments
ARG BINARY_PATH="/usr/local/bin"
ARG APP_USER=vkteams-bot
ARG APP_UID=1001
ARG APP_GID=1001

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    libssl-dev \
    curl \
    binutils \
    ca-certificates

# Create application user and group with home directory
RUN groupadd -g ${APP_GID} ${APP_USER} && \
    useradd -r -u ${APP_UID} -g ${APP_USER} -m -d /home/${APP_USER} ${APP_USER}

# Create application directories
RUN mkdir -p config data logs downloads uploads && \
    chown -R ${APP_USER}:${APP_USER} . && \
    chown -R ${APP_USER}:${APP_USER} ${BINARY_PATH} && \
    chown -R ${APP_USER}:${APP_USER} /home/${APP_USER}

# Copy the binary from builder stage
COPY --chown=${APP_UID}:${APP_GID} --from=builder /app/target/release/vkteams-bot-mcp ${BINARY_PATH}/vkteams-bot-mcp
RUN chmod +x ${BINARY_PATH}/vkteams-bot-mcp

COPY --chown=${APP_UID}:${APP_GID} --from=builder /app/target/release/vkteams-bot-cli ${BINARY_PATH}/vkteams-bot-cli
RUN chmod +x ${BINARY_PATH}/vkteams-bot-cli

# Copy configuration files if they exist
ARG APP_CONFIG_PATH=/app/config/config.toml
COPY --chown=${APP_UID}:${APP_GID} .config/shared-config.toml ${APP_CONFIG_PATH}
RUN chmod +x ${APP_CONFIG_PATH}

# Switch to application user
USER ${APP_USER}

# Set common environment variables
ENV RUST_LOG=info
ENV APP_CONFIG_PATH=${APP_CONFIG_PATH}

# Component-specific environment variables
ARG COMPONENT_TYPE
ENV COMPONENT_TYPE=${COMPONENT_TYPE}

# Set component-specific environment variables
RUN if [ "$COMPONENT_TYPE" = "mcp" ]; then \
    echo "export VKTEAMS_BOT_CLI_PATH=${BINARY_PATH}/vkteams-bot-cli" >> ~/.bashrc; \
    fi

# Create CMD script with variables
RUN echo "exec ${BINARY_PATH}/vkteams-bot-mcp" >> ./cmd.sh && \
    chmod +x ./cmd.sh

HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD ./cmd.sh || exit 1

# Default entrypoint and command using script
CMD ["sh", "-c", "/app/cmd.sh","--version"]

# Labels for better container management
ARG BUILD_DATE=""
ARG BUILD_VERSION=""
ARG BUILD_COMMIT=""

LABEL org.opencontainers.image.title="VKTeams Bot vkteams-bot-mcp" \
    org.opencontainers.image.description="VK Teams Bot vkteams-bot-mcp component" \
    org.opencontainers.image.version="${BUILD_VERSION}" \
    org.opencontainers.image.created="${BUILD_DATE}" \
    org.opencontainers.image.revision="${BUILD_COMMIT}" \
    org.opencontainers.image.source="https://github.com/bug-ops/vkteams-bot" \
    component="${COMPONENT_TYPE}" \
    package="vkteams-bot-mcp"
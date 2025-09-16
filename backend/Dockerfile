# Multi-stage Docker build for AI Orchestrator Hub Backend
# Stage 1: Build stage
FROM rust:1.75-slim AS builder

# Install required dependencies for building
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    sqlite3 \
    libsqlite3-dev \
    && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /app

# Copy Cargo.toml and Cargo.lock first for better caching
COPY Cargo.toml Cargo.lock ./

# Copy source code
COPY src ./src
COPY benches ./benches
COPY tests ./tests

# Build the application in release mode
RUN cargo build --release

# Stage 2: Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    sqlite3 \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN groupadd -r ai-orchestrator && useradd -r -g ai-orchestrator ai-orchestrator

# Create necessary directories
RUN mkdir -p /app/data /app/logs /app/config && \
    chown -R ai-orchestrator:ai-orchestrator /app

# Set working directory
WORKDIR /app

# Copy the compiled binary from builder stage
COPY --from=builder /app/target/release/multiagent-hive /app/multiagent-hive

# Copy configuration files
COPY settings ./settings

# Copy data directory if it exists
COPY data ./data

# Change ownership of the application
RUN chown ai-orchestrator:ai-orchestrator /app/multiagent-hive

# Switch to non-root user
USER ai-orchestrator

# Expose port
EXPOSE 8080

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

# Set environment variables
ENV RUST_LOG=info
ENV DATABASE_URL=/app/data/hive_persistence.db

# Run the application
CMD ["./multiagent-hive"]
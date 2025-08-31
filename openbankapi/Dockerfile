# Multi-stage build for Rust API
FROM rustlang/rust:nightly as builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy Cargo files for dependency caching
COPY Cargo.toml Cargo.lock ./

# Dummy main.rs to build dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs

# Build dependencies
RUN cargo build --release

# Remove dummy main.rs and copy actual source code
RUN rm src/main.rs
COPY src/ ./src/
COPY *.json ./

# Build actual app
RUN cargo build --release

# ----------------------
# Runtime stage
# ----------------------
FROM debian:bookworm-slim

# Install runtime deps
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Non-root user
RUN useradd -m -u 1000 -s /bin/bash appuser
WORKDIR /app

# Copy binary and JSON files
COPY --from=builder /app/target/release/onramptee /app/onramptee
COPY --from=builder /app/*.json /app/

# Copy the .env file directly into image
# Make sure you have a .env in your build context (same folder as Dockerfile)
COPY .env .env

# Change ownership
RUN chown -R appuser:appuser /app
USER appuser

EXPOSE 3000

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:3000/health || exit 1

# Run app
CMD ["./onramptee"]

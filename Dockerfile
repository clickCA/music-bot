# Build stage
FROM rust:1.92 as builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copy manifests first for better caching
COPY Cargo.toml Cargo.lock* ./

# Create a dummy main.rs to pre-build dependencies
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    cargo build --release || true && \
    rm -rf src

# Copy source code
COPY src ./src

# Build the application
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ffmpeg \
    yt-dlp \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copy the binary from builder
COPY --from=builder /app/target/release/music-bot /app/music-bot

# Create temp directory for per-guild downloads
RUN mkdir -p /tmp/music_bot_downloads && \
    chmod 777 /tmp/music_bot_downloads

# Run as non-root user for security
RUN useradd -m -u 1000 -s /bin/bash musicbot && \
    chown -R musicbot:musicbot /app /tmp/music_bot_downloads
USER musicbot
WORKDIR /app

# Set environment variable
ENV DISCORD_TOKEN=""

# Configure STOPSIGNAL for graceful shutdown
# SIGTERM (15) allows graceful shutdown, SIGKILL (9) is force kill
STOPSIGNAL SIGTERM

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=40s --retries=3 \
    CMD test -f /proc/1/cmdline || exit 1

# Run the binary
# exec form (no shell) ensures signals are passed to the process
CMD ["./music-bot"]

# Build stage
FROM rust:1.88-slim AS builder

WORKDIR /usr/src/app

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy cargo files
COPY Cargo.toml Cargo.lock ./

# Copy source code
COPY src ./src

# Build the application
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim AS prod

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    chromium \
    chromium-driver \
    && rm -rf /var/lib/apt/lists/*

# Create app user
RUN useradd -m -u 1001 appuser

# Copy the binary from builder stage
COPY --from=builder /usr/src/app/target/release/llm-web-agent /usr/local/bin/llm-web-agent

# Set permissions
RUN chown appuser:appuser /usr/local/bin/llm-web-agent

# Switch to app user
USER appuser

# Set environment variables for Chrome
ENV CHROME_BIN=/usr/bin/chromium
ENV CHROME_PATH=/usr/bin/chromium

# Expose port
EXPOSE 3000

# Run the application
CMD ["llm-web-agent"] 
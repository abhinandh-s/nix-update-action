# Stage 1: Build Stage (Using a modern Rust version)
FROM rust:1.80-slim AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY . .
RUN cargo build --release

# Stage 2: Final Runtime Stage
FROM debian:bookworm-slim
# Install runtime dependencies for SSL/HTTPS
RUN apt-get update && apt-get install -y ca-certificates libssl-dev && rm -rf /var/lib/apt/lists/*

# Copy the binary from the builder
COPY --from=builder /app/target/release/nix-update-action /usr/local/bin/nix-update-action

ENTRYPOINT ["nix-update-action"]

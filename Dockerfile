# Stage 1: Build Stage (Use 1.83+ to satisfy dependencies)
FROM rust:1.84-slim AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY . .
RUN cargo build --release

# Stage 2: Final Runtime Stage
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates libssl-dev && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/nix-update-action /usr/local/bin/nix-update-action

ENTRYPOINT ["nix-update-action"]

# Build Stage
FROM rust:1.75-slim AS builder
WORKDIR /app
COPY . .
RUN cargo build --release

# Final Stage
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates libssl-dev && rm -rf /var/lib/apt/lists/*
# Change flake-updater to nix-update-action
COPY --from=builder /app/target/release/nix-update-action /usr/local/bin/nix-update-action

ENTRYPOINT ["nix-update-action"]

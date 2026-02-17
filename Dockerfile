# Build Stage
FROM rust:1.75-slim AS builder
WORKDIR /app
COPY . .
RUN cargo build --release

# Final Stage
FROM debian:bookworm-slim
# Install openssl and certificates (needed for reqwest/HTTPS)
RUN apt-get update && apt-get install -y ca-certificates libssl-dev && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/flake-updater /usr/local/bin/flake-updater

ENTRYPOINT ["flake-updater"]

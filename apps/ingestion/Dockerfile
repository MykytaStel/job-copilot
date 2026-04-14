# Build context: monorepo root (needs apps/engine-api/migrations at compile time)
FROM rust:slim AS builder
RUN apt-get update && apt-get install -y pkg-config && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY . .
RUN cargo build --release --manifest-path apps/ingestion/Cargo.toml

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /app/apps/ingestion/target/release/ingestion .
ENTRYPOINT ["./ingestion"]

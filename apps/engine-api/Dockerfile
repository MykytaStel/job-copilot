# ── Stage 1: build ───────────────────────────────────────────────────────────
FROM rust:slim AS builder

RUN apt-get update && apt-get install -y pkg-config && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY . .
RUN cargo build --release

# ── Stage 2: runtime ─────────────────────────────────────────────────────────
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /app/target/release/engine-api .
COPY --from=builder /app/migrations ./migrations

ENV PORT=8080
ENV RUN_DB_MIGRATIONS=true

EXPOSE 8080
CMD ["./engine-api"]

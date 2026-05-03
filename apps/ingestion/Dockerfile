# Build context: monorepo root
# This Dockerfile expects to be built from the repository root:
# docker build -f apps/ingestion/Dockerfile .

# ── Stage 1: build ───────────────────────────────────────────────────────────
FROM rust:1-slim-bookworm AS builder

RUN apt-get update \
	&& apt-get install -y --no-install-recommends pkg-config \
	&& rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY . .

RUN cargo build --release -p ingestion

# ── Stage 2: runtime ─────────────────────────────────────────────────────────
FROM debian:bookworm-slim

RUN apt-get update \
	&& apt-get install -y --no-install-recommends ca-certificates \
	&& rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/ingestion ./ingestion

ENTRYPOINT ["./ingestion"]
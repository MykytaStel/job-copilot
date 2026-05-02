# Build context: monorepo root
# This Dockerfile expects to be built from the repository root:
# docker build -f apps/engine-api/Dockerfile .

# ── Stage 1: build ───────────────────────────────────────────────────────────
FROM rust:slim AS builder

RUN apt-get update \
	&& apt-get install -y --no-install-recommends pkg-config \
	&& rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY . .

RUN cargo build --release -p engine-api

# ── Stage 2: runtime ─────────────────────────────────────────────────────────
FROM debian:bookworm-slim

RUN apt-get update \
	&& apt-get install -y --no-install-recommends ca-certificates curl \
	&& rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/engine-api ./engine-api
COPY --from=builder /app/apps/engine-api/migrations ./migrations

ENV PORT=8080
ENV RUN_DB_MIGRATIONS=true

EXPOSE 8080

CMD ["./engine-api"]
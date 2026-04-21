#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
COMPOSE_FILE="$ROOT_DIR/infra/docker-compose.postgres.yml"
TMP_DIR="$ROOT_DIR/.tmp"
ENGINE_LOG="$TMP_DIR/engine-api-local.log"

PORT="${PORT:-18080}"
PGHOST="${PGHOST:-127.0.0.1}"
PGPORT="${PGPORT:-5432}"
PGUSER="${PGUSER:-jobcopilot}"
PGPASSWORD="${PGPASSWORD:-jobcopilot}"
PGDATABASE="${PGDATABASE:-jobcopilot}"
DATABASE_URL="${DATABASE_URL:-postgres://${PGUSER}:${PGPASSWORD}@${PGHOST}:${PGPORT}/${PGDATABASE}}"
INPUT_PATH="${INPUT_PATH:-apps/ingestion/examples/mock_source_jobs.json}"
INPUT_FORMAT="${INPUT_FORMAT:-mock-source}"
RUN_SECOND_PASS="${RUN_SECOND_PASS:-1}"
SECOND_INPUT_PATH="${SECOND_INPUT_PATH:-apps/ingestion/examples/mock_source_jobs_updated.json}"
RUN_THIRD_PASS="${RUN_THIRD_PASS:-1}"
THIRD_INPUT_PATH="${THIRD_INPUT_PATH:-apps/ingestion/examples/mock_source_jobs_reactivated.json}"
KEEP_RUNNING="${KEEP_RUNNING:-1}"

ENGINE_PID=""

require_command() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "Missing required command: $1" >&2
    exit 1
  fi
}

compose_cmd() {
  if command -v docker >/dev/null 2>&1 && docker compose version >/dev/null 2>&1; then
    docker compose -f "$COMPOSE_FILE" "$@"
    return
  fi

  if command -v docker-compose >/dev/null 2>&1; then
    docker-compose -f "$COMPOSE_FILE" "$@"
    return
  fi

  echo "Docker Compose is required." >&2
  exit 1
}

cleanup() {
  if [[ -n "$ENGINE_PID" ]] && kill -0 "$ENGINE_PID" >/dev/null 2>&1; then
    kill "$ENGINE_PID" >/dev/null 2>&1 || true
    wait "$ENGINE_PID" >/dev/null 2>&1 || true
  fi

  compose_cmd down -v >/dev/null 2>&1 || true
}

fail() {
  echo "Local ingestion demo failed: $1" >&2
  if [[ -f "$ENGINE_LOG" ]]; then
    echo "" >&2
    echo "Last engine-api log lines:" >&2
    tail -n 40 "$ENGINE_LOG" >&2 || true
  fi
  exit 1
}

wait_for_postgres() {
  local attempts=0
  until compose_cmd exec -T postgres pg_isready -U "$PGUSER" -d "$PGDATABASE" >/dev/null 2>&1; do
    attempts=$((attempts + 1))
    if [[ "$attempts" -ge 60 ]]; then
      fail "postgres container did not become ready in time"
    fi
    sleep 1
  done
}

wait_for_engine() {
  local attempts=0
  until curl -sS "http://127.0.0.1:${PORT}/health" >/dev/null 2>&1; do
    attempts=$((attempts + 1))
    if [[ -n "$ENGINE_PID" ]] && ! kill -0 "$ENGINE_PID" >/dev/null 2>&1; then
      fail "engine-api exited before becoming healthy"
    fi
    if [[ "$attempts" -ge 60 ]]; then
      fail "engine-api did not become healthy in time"
    fi
    sleep 1
  done
}

run_ingestion() {
  local input_path="$1"
  echo "Running ingestion for ${input_path}..."
  (
    cd "$ROOT_DIR"
    DATABASE_URL="$DATABASE_URL" cargo run --features mock --manifest-path apps/ingestion/Cargo.toml -- --input "$input_path" --input-format "$INPUT_FORMAT"
  )
}

trap cleanup EXIT

require_command cargo
require_command curl
require_command jq
require_command bash
require_command lsof

if ! command -v docker >/dev/null 2>&1 && ! command -v docker-compose >/dev/null 2>&1; then
  echo "Docker is required for the local ingestion demo." >&2
  exit 1
fi

if lsof -iTCP:"$PORT" -sTCP:LISTEN -n -P >/dev/null 2>&1; then
  echo "Port $PORT is already in use. Run with a different PORT, for example: PORT=18081 pnpm local:ingestion:demo" >&2
  exit 1
fi

mkdir -p "$TMP_DIR"

echo "Starting Postgres..."
compose_cmd up -d
wait_for_postgres

echo "Starting engine-api on http://127.0.0.1:${PORT} ..."
(
  cd "$ROOT_DIR"
  PORT="$PORT" DATABASE_URL="$DATABASE_URL" RUN_DB_MIGRATIONS=true \
    cargo run --manifest-path apps/engine-api/Cargo.toml
) >"$ENGINE_LOG" 2>&1 &
ENGINE_PID="$!"

wait_for_engine

run_ingestion "$INPUT_PATH"

if [[ "$RUN_SECOND_PASS" == "1" ]]; then
  run_ingestion "$SECOND_INPUT_PATH"
fi

if [[ "$RUN_THIRD_PASS" == "1" ]]; then
  run_ingestion "$THIRD_INPUT_PATH"
fi

echo ""
echo "Local stack is ready."
echo "Health: http://127.0.0.1:${PORT}/health"
echo "Search demo: http://127.0.0.1:${PORT}/api/v1/search?q=SignalHire"
echo "Jobs feed demo: http://127.0.0.1:${PORT}/api/v1/jobs/recent"
echo "Engine log: $ENGINE_LOG"
echo ""
echo "Current job lifecycle rows:"
compose_cmd exec -T postgres psql -U "$PGUSER" -d "$PGDATABASE" -P pager=off -c \
  "SELECT id, is_active, first_seen_at, last_seen_at, inactivated_at, reactivated_at FROM jobs ORDER BY last_seen_at DESC;"
echo ""
echo "Current job_variants rows:"
compose_cmd exec -T postgres psql -U "$PGUSER" -d "$PGDATABASE" -P pager=off -c \
  "SELECT source, source_job_id, job_id, is_active, last_seen_at, inactivated_at, LEFT(raw_hash, 12) AS raw_hash_prefix, fetched_at FROM job_variants ORDER BY source, source_job_id;"
echo ""
echo "API summary:"
curl -sS "http://127.0.0.1:${PORT}/api/v1/jobs/recent" | jq '{summary, jobs: [.jobs[] | {id, lifecycle_stage, is_active, source: .primary_variant.source}]}'
echo ""

if [[ "$KEEP_RUNNING" == "1" ]]; then
  echo "Press Ctrl-C to stop engine-api and remove the local Postgres volume."
  wait "$ENGINE_PID"
else
  echo "KEEP_RUNNING=0, shutting the local stack down."
fi

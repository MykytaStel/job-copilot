#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
COMPOSE_FILE="${COMPOSE_FILE:-$ROOT_DIR/infra/docker-compose.yml}"
API_BASE_URL="${API_BASE_URL:-http://127.0.0.1:8080}"
DATABASE_URL="${DATABASE_URL:-postgres://jobcopilot:jobcopilot@127.0.0.1:5432/jobcopilot}"
INPUT_PATH="${INPUT_PATH:-apps/ingestion/tests/fixtures/mock_source_jobs_initial.json}"
INPUT_FORMAT="${INPUT_FORMAT:-mock-source}"
COMPOSE_CLEANUP="${COMPOSE_CLEANUP:-1}"

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
  if [[ "$COMPOSE_CLEANUP" == "1" ]]; then
    compose_cmd down -v >/dev/null 2>&1 || true
  fi
}

fail() {
  echo "E2E smoke failed: $1" >&2
  echo "" >&2
  echo "Last engine-api log lines:" >&2
  compose_cmd logs --tail=80 engine-api >&2 || true
  exit 1
}

json_field() {
  local field="$1"
  python3 -c "import json, sys; print(json.load(sys.stdin)['$field'])"
}

wait_for_api() {
  local attempts=0
  until curl -fsS "$API_BASE_URL/ready" >/dev/null 2>&1; do
    attempts=$((attempts + 1))
    if [[ "$attempts" -ge 120 ]]; then
      fail "engine-api did not become ready at $API_BASE_URL"
    fi
    sleep 1
  done
}

extract_first_job_id() {
  python3 -c '
import json
import sys

payload = json.load(sys.stdin)
jobs = payload.get("jobs", [])
if not jobs:
    raise SystemExit("jobs feed returned no jobs")
print(jobs[0]["id"])
'
}

assert_saved_feedback() {
  local expected_job_id="$1"
  local feedback_json="$2"
  python3 - "$expected_job_id" "$feedback_json" <<'PY'
import json
import sys

expected_job_id = sys.argv[1]
payload = json.loads(sys.argv[2])
summary = payload.get("summary", {})
jobs = payload.get("jobs", [])

if summary.get("saved_jobs_count", 0) < 1:
    raise SystemExit("feedback summary did not record a saved job")

if not any(job.get("job_id") == expected_job_id and job.get("saved") is True for job in jobs):
    raise SystemExit(f"saved feedback for {expected_job_id} was not returned")
PY
}

trap cleanup EXIT

require_command cargo
require_command curl
require_command python3

echo "Starting Docker Compose services..."
if [[ "$COMPOSE_CLEANUP" == "1" ]]; then
  compose_cmd down -v --remove-orphans >/dev/null 2>&1 || true
fi
compose_cmd up --build -d postgres engine-api
wait_for_api

echo "Creating smoke profile..."
profile_email="smoke-$(date +%s)-${RANDOM}@example.test"
auth_response="$(
  curl -fsS \
    -H "Content-Type: application/json" \
    -X POST "$API_BASE_URL/api/v1/auth/register" \
    --data @- <<JSON
{"name":"Smoke Candidate","email":"$profile_email","password":"smoke-password-123","raw_text":"Senior backend and data engineer with Rust, PostgreSQL, ingestion pipelines, search ranking, and analytics experience."}
JSON
)"
token="$(printf '%s' "$auth_response" | json_field token)"
profile_id="$(printf '%s' "$auth_response" | json_field profile_id)"

echo "Running ingestion fixture..."
(
  cd "$ROOT_DIR"
  DATABASE_URL="$DATABASE_URL" RUN_DB_MIGRATIONS=false \
    cargo run --features mock --manifest-path apps/ingestion/Cargo.toml -- \
      --input "$INPUT_PATH" \
      --input-format "$INPUT_FORMAT"
)

echo "Checking jobs feed..."
feed_response="$(
  curl -fsS \
    -H "Authorization: Bearer $token" \
    "$API_BASE_URL/api/v1/jobs/recent?profile_id=$profile_id&limit=10&lifecycle=active"
)"
job_id="$(printf '%s' "$feed_response" | extract_first_job_id)"

echo "Saving job $job_id..."
save_response="$(
  curl -fsS \
    -H "Authorization: Bearer $token" \
    -X PUT "$API_BASE_URL/api/v1/profiles/$profile_id/jobs/$job_id/saved"
)"
python3 - "$job_id" "$save_response" <<'PY'
import json
import sys

expected_job_id = sys.argv[1]
payload = json.loads(sys.argv[2])
if payload.get("job_id") != expected_job_id or payload.get("saved") is not True:
    raise SystemExit("save endpoint did not return saved feedback")
PY

echo "Checking feedback endpoint..."
feedback_response="$(
  curl -fsS \
    -H "Authorization: Bearer $token" \
    "$API_BASE_URL/api/v1/profiles/$profile_id/feedback"
)"
assert_saved_feedback "$job_id" "$feedback_response"

echo "E2E smoke passed: profile=$profile_id job=$job_id"

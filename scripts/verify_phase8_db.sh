#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
COMPOSE_FILE="$ROOT_DIR/infra/docker-compose.postgres.yml"
TMP_DIR="$ROOT_DIR/.tmp"
ENGINE_LOG="$TMP_DIR/engine-api-phase8-db.log"

PORT="${PORT:-18080}"
PGHOST="${PGHOST:-127.0.0.1}"
PGPORT="${PGPORT:-5432}"
PGUSER="${PGUSER:-jobcopilot}"
PGPASSWORD="${PGPASSWORD:-jobcopilot}"
PGDATABASE="${PGDATABASE:-jobcopilot}"
KEEP_POSTGRES="${KEEP_POSTGRES:-0}"
DATABASE_URL="${DATABASE_URL:-postgres://${PGUSER}:${PGPASSWORD}@${PGHOST}:${PGPORT}/${PGDATABASE}}"
BASE_URL="http://127.0.0.1:${PORT}"
RUN_ID="$(date +%s)"
TEST_JOB_ID="job_phase8_verify_${RUN_ID}"
TEST_CONTACT_EMAIL="phase8.${RUN_ID}@example.com"
BEFORE_TERM="beforetoken${RUN_ID}"
AFTER_TERM="aftertoken${RUN_ID}"

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

  echo "Docker Compose is required. Install Docker Desktop or docker-compose first." >&2
  exit 1
}

cleanup() {
  if [[ -n "$ENGINE_PID" ]] && kill -0 "$ENGINE_PID" >/dev/null 2>&1; then
    kill "$ENGINE_PID" >/dev/null 2>&1 || true
    wait "$ENGINE_PID" >/dev/null 2>&1 || true
  fi

  if [[ "$KEEP_POSTGRES" != "1" ]]; then
    compose_cmd down -v >/dev/null 2>&1 || true
  fi
}

fail() {
  echo "Verification failed: $1" >&2
  if [[ -f "$ENGINE_LOG" ]]; then
    echo "" >&2
    echo "Last engine-api log lines:" >&2
    tail -n 40 "$ENGINE_LOG" >&2 || true
  fi
  exit 1
}

assert_json() {
  local payload="$1"
  local message="$2"
  shift 2

  if ! printf '%s' "$payload" | jq -e "$@" >/dev/null; then
    echo "$payload" | jq . >&2 || echo "$payload" >&2
    fail "$message"
  fi
}

api_request() {
  local method="$1"
  local path="$2"
  local body="${3-}"
  local response_file
  response_file="$(mktemp)"

  local http_code
  if [[ -n "$body" ]]; then
    http_code="$(
      curl -sS -o "$response_file" -w '%{http_code}' \
        -X "$method" \
        -H 'Content-Type: application/json' \
        --data "$body" \
        "$BASE_URL$path"
    )"
  else
    http_code="$(
      curl -sS -o "$response_file" -w '%{http_code}' \
        -X "$method" \
        "$BASE_URL$path"
    )"
  fi

  if [[ "$http_code" -lt 200 || "$http_code" -ge 300 ]]; then
    cat "$response_file" >&2
    rm -f "$response_file"
    fail "API request failed: ${method} ${path} (${http_code})"
  fi

  cat "$response_file"
  rm -f "$response_file"
}

psql_query() {
  local sql="$1"
  compose_cmd exec -T postgres psql -U "$PGUSER" -d "$PGDATABASE" -tA -c "$sql"
}

wait_for_engine() {
  local attempts=0
  until curl -sS "$BASE_URL/health" >/dev/null 2>&1; do
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

trap cleanup EXIT

require_command cargo
require_command curl
require_command jq
require_command bash
require_command lsof
if ! command -v docker >/dev/null 2>&1 && ! command -v docker-compose >/dev/null 2>&1; then
  echo "Docker is required for DB-backed verification. Install Docker Desktop first." >&2
  exit 1
fi

if lsof -iTCP:"$PORT" -sTCP:LISTEN -n -P >/dev/null 2>&1; then
  echo "Port $PORT is already in use. Stop the existing process or run with a different PORT, for example: PORT=18081 pnpm verify:phase8:db" >&2
  exit 1
fi

mkdir -p "$TMP_DIR"

echo "Starting Postgres container..."
compose_cmd up -d
wait_for_postgres

echo "Starting engine-api with DATABASE_URL=$DATABASE_URL"
(
  cd "$ROOT_DIR"
  PORT="$PORT" DATABASE_URL="$DATABASE_URL" RUN_DB_MIGRATIONS=true \
    cargo run --manifest-path apps/engine-api/Cargo.toml
) >"$ENGINE_LOG" 2>&1 &
ENGINE_PID="$!"

wait_for_engine

health_response="$(api_request GET /health)"
assert_json \
  "$health_response" \
  "health endpoint did not report a migrated, healthy database" \
  '.status == "ok" and .database.status == "ok" and .database.configured == true and .database.migrations_enabled_on_startup == true'

echo "Seeding demo data..."
compose_cmd exec -T postgres psql -U "$PGUSER" -d "$PGDATABASE" < "$ROOT_DIR/apps/engine-api/seeds/dev.sql"

null_backfill_count="$(psql_query "SELECT COUNT(*) FROM jobs WHERE search_vector IS NULL;")"
[[ "$null_backfill_count" == "0" ]] || fail "search_vector backfill left NULL values in jobs"

echo "Verifying search_vector trigger on insert and update..."
psql_query "
INSERT INTO jobs (
  id,
  title,
  company_name,
  location,
  remote_type,
  seniority,
  description_text,
  posted_at,
  last_seen_at,
  is_active
) VALUES (
  '${TEST_JOB_ID}',
  'Phase 8 Verification Engineer ${BEFORE_TERM}',
  'TriggerWorks',
  'Kyiv',
  'remote',
  'senior',
  'search vector insert verification ${BEFORE_TERM}',
  NOW(),
  NOW(),
  TRUE
);
" >/dev/null

insert_trigger_match="$(psql_query "SELECT search_vector @@ websearch_to_tsquery('simple', '${BEFORE_TERM}') FROM jobs WHERE id = '${TEST_JOB_ID}';")"
[[ "$insert_trigger_match" == "t" ]] || fail "search_vector trigger did not populate on insert"

psql_query "
UPDATE jobs
SET
  title = 'Phase 8 Verification Engineer ${AFTER_TERM}',
  description_text = 'search vector update verification ${AFTER_TERM}'
WHERE id = '${TEST_JOB_ID}';
" >/dev/null

before_after_update="$(psql_query "SELECT search_vector @@ websearch_to_tsquery('simple', '${BEFORE_TERM}') FROM jobs WHERE id = '${TEST_JOB_ID}';")"
after_after_update="$(psql_query "SELECT search_vector @@ websearch_to_tsquery('simple', '${AFTER_TERM}') FROM jobs WHERE id = '${TEST_JOB_ID}';")"
[[ "$before_after_update" == "f" ]] || fail "search_vector still matched the old token after update"
[[ "$after_after_update" == "t" ]] || fail "search_vector did not update for the new token"

echo "Creating a real application against Postgres..."
application_response="$(
  api_request POST /api/v1/applications \
    "{\"job_id\":\"${TEST_JOB_ID}\",\"status\":\"saved\"}"
)"
application_id="$(printf '%s' "$application_response" | jq -r '.id')"
[[ -n "$application_id" && "$application_id" != "null" ]] || fail "create application did not return an id"

note_response="$(
  api_request POST "/api/v1/applications/${application_id}/notes" \
    '{"content":"Phase 8 smoke note"}'
)"
assert_json "$note_response" "note creation did not return the expected content" '.content == "Phase 8 smoke note"'

contact_response="$(
  api_request POST /api/v1/contacts \
    "{\"name\":\"Phase 8 Recruiter\",\"email\":\"${TEST_CONTACT_EMAIL}\",\"company\":\"TriggerWorks\",\"role\":\"Recruiter\"}"
)"
contact_id="$(printf '%s' "$contact_response" | jq -r '.id')"
[[ -n "$contact_id" && "$contact_id" != "null" ]] || fail "create contact did not return an id"

contacts_list="$(api_request GET /api/v1/contacts)"
assert_json "$contacts_list" "contact list did not include the created contact" --arg email "$TEST_CONTACT_EMAIL" '.contacts | any(.email == $email)'

linked_contact="$(
  api_request POST "/api/v1/applications/${application_id}/contacts" \
    "{\"contact_id\":\"${contact_id}\",\"relationship\":\"recruiter\"}"
)"
assert_json "$linked_contact" "application contact link failed" --arg contact_id "$contact_id" '.contact.id == $contact_id and .relationship == "recruiter"'

offer_response="$(
  api_request PUT "/api/v1/applications/${application_id}/offer" \
    '{"status":"received","compensation_min":6500,"compensation_max":7800,"compensation_currency":"USD","starts_at":"2026-05-01T09:00:00Z","notes":"Phase 8 offer"}'
)"
assert_json "$offer_response" "offer upsert failed" '.status == "received" and .compensation_currency == "USD"'

patch_response="$(
  api_request PATCH "/api/v1/applications/${application_id}" \
    '{"status":"offer","due_date":"2026-05-10T12:00:00Z"}'
)"
assert_json "$patch_response" "application patch did not persist status and due_date" '.status == "offer" and (.due_date | startswith("2026-05-10"))'

detail_response="$(api_request GET "/api/v1/applications/${application_id}")"
assert_json "$detail_response" "application detail did not return the patched status" '.status == "offer"'
assert_json "$detail_response" "application detail did not return the patched due date" '(.due_date | startswith("2026-05-10"))'
assert_json "$detail_response" "application detail did not include the created note" '.notes | any(.content == "Phase 8 smoke note")'
assert_json "$detail_response" "application detail did not include the linked contact" --arg contact_id "$contact_id" '.contacts | any(.contact.id == $contact_id and .relationship == "recruiter")'
assert_json "$detail_response" "application detail did not include the offer" '.offer.status == "received" and .offer.notes == "Phase 8 offer"'

clear_due_date_response="$(
  api_request PATCH "/api/v1/applications/${application_id}" \
    '{"due_date":null}'
)"
assert_json "$clear_due_date_response" "application patch did not clear due_date" '.due_date == null'

detail_after_clear="$(api_request GET "/api/v1/applications/${application_id}")"
assert_json "$detail_after_clear" "application detail did not reflect cleared due_date" '.status == "offer" and .due_date == null'

echo "Verifying search endpoints..."
search_token_response="$(api_request GET "/api/v1/search?q=${AFTER_TERM}")"
assert_json "$search_token_response" "search did not find the trigger-updated job" --arg job_id "$TEST_JOB_ID" '.jobs | any(.id == $job_id)'

search_page_one="$(api_request GET "/api/v1/search?q=engineer&limit=2&page=1")"
assert_json "$search_page_one" "search page 1 pagination check failed" '.page == 1 and .per_page == 2 and (.jobs | length) == 2 and .has_more == true'

search_page_two="$(api_request GET "/api/v1/search?q=engineer&limit=2&page=2")"
assert_json "$search_page_two" "search page 2 pagination check failed" '.page == 2 and (.jobs | length) >= 1'

echo ""
echo "Phase 8 DB verification passed."
echo "Engine log: $ENGINE_LOG"
if [[ "$KEEP_POSTGRES" == "1" ]]; then
  echo "Postgres is still running because KEEP_POSTGRES=1."
fi

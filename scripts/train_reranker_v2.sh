#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
ML_DIR="$ROOT_DIR/apps/ml"
DATASET_PATH="${DATASET_PATH:-$ML_DIR/reranker-dataset.json}"
MODEL_PATH="${MODEL_PATH:-$ML_DIR/models/trained-reranker-v3.json}"
REPORT_DIR="${REPORT_DIR:-$ML_DIR/reports}"
CANDIDATE_MODEL_PATH="$ML_DIR/models/.trained-reranker-v3.candidate.json"
TRAINING_REPORT_PATH="$REPORT_DIR/trained-reranker-v3-training.json"
EVALUATION_REPORT_PATH="$REPORT_DIR/trained-reranker-v3-evaluation.json"

ENGINE_API_BASE_URL="${ENGINE_API_BASE_URL:-http://localhost:8080}"
PROFILE_ID="${PROFILE_ID:-}"
TOP_N="${TOP_N:-10}"
EPOCHS="${EPOCHS:-500}"
LEARNING_RATE="${LEARNING_RATE:-0.08}"
L2="${L2:-0.01}"
MAX_SCORE_DELTA="${MAX_SCORE_DELTA:-8}"
MIN_EXAMPLES="${MIN_EXAMPLES:-4}"
MIN_POSITIVE="${MIN_POSITIVE:-1}"
MIN_MEDIUM="${MIN_MEDIUM:-0}"
MIN_NEGATIVE="${MIN_NEGATIVE:-1}"
APPLY_TO_DOCKER="${APPLY_TO_DOCKER:-0}"

require_command() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "Missing required command: $1" >&2
    exit 1
  fi
}

json_count() {
  local query="$1"
  jq -r "$query" "$DATASET_PATH"
}

if [[ -z "$PROFILE_ID" ]]; then
  echo "PROFILE_ID is required. Example: PROFILE_ID=<profile-id> pnpm train:reranker:v3" >&2
  exit 1
fi

require_command curl
require_command jq

if [[ -x "$ML_DIR/.venv/bin/python" ]]; then
  PYTHON_BIN="$ML_DIR/.venv/bin/python"
else
  require_command python3
  PYTHON_BIN="python3"
fi

mkdir -p "$(dirname "$DATASET_PATH")" "$(dirname "$MODEL_PATH")" "$REPORT_DIR"

echo "Exporting reranker dataset for profile $PROFILE_ID..."
curl -fsS "$ENGINE_API_BASE_URL/api/v1/profiles/$PROFILE_ID/reranker-dataset" \
  > "$DATASET_PATH"

example_count="$(json_count '.examples | length')"
positive_count="$(json_count '[.examples[] | select(.label == "positive")] | length')"
medium_count="$(json_count '[.examples[] | select(.label == "medium")] | length')"
negative_count="$(json_count '[.examples[] | select(.label == "negative")] | length')"

echo "Dataset labels: examples=$example_count positive=$positive_count medium=$medium_count negative=$negative_count"

if (( example_count < MIN_EXAMPLES )); then
  echo "Not enough examples: got $example_count, need at least $MIN_EXAMPLES." >&2
  exit 1
fi
if (( positive_count < MIN_POSITIVE )); then
  echo "Not enough positive examples: got $positive_count, need at least $MIN_POSITIVE." >&2
  exit 1
fi
if (( medium_count < MIN_MEDIUM )); then
  echo "Not enough medium examples: got $medium_count, need at least $MIN_MEDIUM." >&2
  exit 1
fi
if (( negative_count < MIN_NEGATIVE )); then
  echo "Not enough negative examples: got $negative_count, need at least $MIN_NEGATIVE." >&2
  exit 1
fi

echo "Training candidate model..."
(
  cd "$ML_DIR"
  "$PYTHON_BIN" -m app.trained_reranker \
    "$DATASET_PATH" \
    --output "$CANDIDATE_MODEL_PATH" \
    --top-n "$TOP_N" \
    --epochs "$EPOCHS" \
    --learning-rate "$LEARNING_RATE" \
    --l2 "$L2" \
    --max-score-delta "$MAX_SCORE_DELTA"
) | tee "$TRAINING_REPORT_PATH"

echo "Evaluating candidate model..."
(
  cd "$ML_DIR"
  "$PYTHON_BIN" -m app.reranker_evaluation \
    "$DATASET_PATH" \
    --trained-model "$CANDIDATE_MODEL_PATH" \
    --top-n "$TOP_N"
) | tee "$EVALUATION_REPORT_PATH"

cp "$CANDIDATE_MODEL_PATH" "$MODEL_PATH"
rm -f "$CANDIDATE_MODEL_PATH"

echo "Promoted trained reranker model: $MODEL_PATH"
echo "Training report: $TRAINING_REPORT_PATH"
echo "Evaluation report: $EVALUATION_REPORT_PATH"

if [[ "$APPLY_TO_DOCKER" == "1" ]]; then
  require_command docker
  echo "Restarting Docker engine-api with trained reranker enabled..."
  (
    cd "$ROOT_DIR"
    TRAINED_RERANKER_ENABLED=true \
      docker compose -f infra/docker-compose.yml up -d --build engine-api
  )
fi

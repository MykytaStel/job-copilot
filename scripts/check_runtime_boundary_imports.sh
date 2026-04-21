#!/usr/bin/env bash
set -euo pipefail

MODE="${1:-report}"

TARGETS=(
  "apps/web/src"
  "apps/ml/app"
  "apps/engine-api/src"
  "apps/ingestion/src"
)

GLOBS=(
  "!**/tests/**"
  "!**/test/**"
  "!**/*.test.*"
  "!**/*_test.py"
  "!**/__pycache__/**"
)

PATTERN="from ['\"][^'\"]*(tests|examples|fixtures|seeds)[^'\"]*['\"]|import .*['\"][^'\"]*(tests|examples|fixtures|seeds)[^'\"]*['\"]|require\\(['\"][^'\"]*(tests|examples|fixtures|seeds)[^'\"]*['\"]\\)|include_(str|bytes)!\\([^)]*(tests|examples|fixtures|seeds)[^)]*\\)"

FOUND=0
RG_ARGS=(-n "$PATTERN")

for glob in "${GLOBS[@]}"; do
  RG_ARGS+=(-g "$glob")
done

for target in "${TARGETS[@]}"; do
  if [ -d "$target" ]; then
    if rg "${RG_ARGS[@]}" "$target"; then
      FOUND=1
    fi
  fi
done

if [ "$FOUND" -ne 0 ]; then
  echo ""
  echo "Runtime boundary warning: runtime paths should not depend on tests/examples/fixtures/seeds."
  if [ "$MODE" = "fail" ]; then
    exit 1
  fi
fi

if [ "$FOUND" -eq 0 ]; then
  echo "Runtime boundary import guard passed."
else
  echo "Runtime boundary import guard reported warnings."
fi

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

is_rust_test_module_match() {
  local file="$1"
  local target_line="$2"

  [[ "$file" == *.rs ]] || return 1

  awk -v target_line="$target_line" '
    NR >= target_line { exit }
    /#\[cfg\(test\)\]/ { cfg_line = NR }
    cfg_line && NR <= cfg_line + 2 && /mod tests[[:space:]]*\{/ { in_test_module = 1 }
    END { exit(in_test_module ? 0 : 1) }
  ' "$file"
}

for target in "${TARGETS[@]}"; do
  if [ ! -d "$target" ]; then
    continue
  fi

  while IFS=: read -r file line match; do
    if [[ "$match" == *include_str* || "$match" == *include_bytes* ]] \
      && is_rust_test_module_match "$file" "$line"; then
      continue
    fi

    printf '%s:%s:%s\n' "$file" "$line" "$match"
    FOUND=1
  done < <(rg "${RG_ARGS[@]}" "$target" || true)
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

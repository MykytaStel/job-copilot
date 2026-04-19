#!/usr/bin/env bash
set -euo pipefail

ROOT="${1:-apps/web/src}"

TARGETS=(
  "$ROOT/pages"
  "$ROOT/features"
  "$ROOT/components"
  "$ROOT/lib"
  "$ROOT/App.tsx"
  "$ROOT/AppShellNew.tsx"
  "$ROOT/Layout.tsx"
)

PATTERN="from ['\"](\.\.?/)+api['\"]|from ['\"][^'\"]*/api['\"]"

FOUND=0

for target in "${TARGETS[@]}"; do
  if [ -e "$target" ]; then
    if rg -n "$PATTERN" "$target"; then
      FOUND=1
    fi
  fi
done

if [ "$FOUND" -ne 0 ]; then
  echo ""
  echo "Direct barrel imports from src/api.ts are not allowed inside web internals."
  echo "Use domain modules such as:"
  echo "  ../api/jobs"
  echo "  ../api/profiles"
  echo "  ../api/enrichment"
  echo "  ../api/feedback"
  echo "  ../api/notifications"
  exit 1
fi

echo "Web API barrel import guard passed."
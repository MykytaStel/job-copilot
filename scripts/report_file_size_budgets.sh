#!/usr/bin/env bash
set -euo pipefail

THRESHOLD="${1:-400}"

echo "Reporting runtime source files over ${THRESHOLD} lines"

find apps packages \
  -path '*/src/*' \
  -type f \
  \( -name '*.ts' -o -name '*.tsx' -o -name '*.rs' -o -name '*.py' \) \
  ! -path '*/tests/*' \
  ! -name '*.test.*' \
  ! -path '*/__pycache__/*' \
  -print0 \
  | xargs -0 wc -l \
  | awk -v threshold="$THRESHOLD" '$1 > threshold { print }' \
  | sort -nr

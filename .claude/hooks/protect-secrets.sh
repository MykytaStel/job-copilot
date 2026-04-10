#!/usr/bin/env bash
set -euo pipefail
input="$(cat)"
if echo "$input" | grep -Eiq '\.env|secret|token|private[_-]?key'; then
  echo '{"decision":"ask","reason":"This change may touch secrets or environment configuration. Verify before applying."}'
  exit 0
fi

echo '{"decision":"approve"}'

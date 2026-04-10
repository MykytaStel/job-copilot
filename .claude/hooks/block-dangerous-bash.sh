#!/usr/bin/env bash
set -euo pipefail
input="$(cat)"
if echo "$input" | grep -Eiq 'rm -rf /|sudo rm -rf|mkfs|dd if='; then
  echo '{"decision":"deny","reason":"Blocked potentially destructive shell command."}'
  exit 0
fi

echo '{"decision":"approve"}'

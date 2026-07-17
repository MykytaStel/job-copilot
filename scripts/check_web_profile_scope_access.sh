#!/usr/bin/env bash
set -euo pipefail

ROOT="${1:-apps/web/src}"

FOUND=0

if rg -n "engine_api_profile_id|PROFILE_ID_STORAGE_KEY" \
  "$ROOT" \
  --glob '!**/*.test.*' \
  --glob '!**/tests/**' \
  --glob '!**/lib/profileSession.ts'
then
  FOUND=1
fi

if [ "$FOUND" -ne 0 ]; then
  echo ""
  echo "Profile id storage must stay centralized in apps/web/src/lib/profileSession.ts."
  exit 1
fi

echo "Web profile scope guard passed."

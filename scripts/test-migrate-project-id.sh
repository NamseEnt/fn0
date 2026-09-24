#!/usr/bin/env bash

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${REPO_ROOT}"

python3 scripts/test-migrate-project-id.py

if output="$(scripts/migrate-project-id.sh apply --old-id namse-mottomite --new-id c9qxk46r 2>&1)"; then
  echo "project ID migration apply accepted a missing --apply flag" >&2
  exit 1
fi
[[ "${output}" == *"explicit --apply flag"* ]]

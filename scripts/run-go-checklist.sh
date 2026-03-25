#!/usr/bin/env bash

set -euo pipefail

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
go_dir="$root_dir/go"

echo "[go-checklist] checking formatting"
unformatted="$(find "$go_dir" -type f -name '*.go' -print0 | xargs -0 gofmt -l)"
if [[ -n "$unformatted" ]]; then
  echo "[go-checklist] gofmt found unformatted files:" >&2
  printf '%s\n' "$unformatted" >&2
  exit 1
fi

echo "[go-checklist] running go test ./... -count=1"
(cd "$go_dir" && go test ./... -count=1)

echo "[go-checklist] ok"

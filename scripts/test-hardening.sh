#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."
cargo test --workspace --all-targets
cargo test --workspace --doc

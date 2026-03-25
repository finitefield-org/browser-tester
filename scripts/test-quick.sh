#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."
cargo test -p browser_tester --tests

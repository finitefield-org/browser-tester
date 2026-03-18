#!/usr/bin/env bash

set -euo pipefail

usage() {
  cat <<'EOF'
Usage:
  scripts/run-test-layer.sh <layer> [cargo-test-args...]

Layers:
  contract        Run minimal public contract tests
  contract-build  Build the public contract test target without running it
  subsystem       Run internal lib tests under src/tests
  integration     Run grouped integration regressions
  fuzz            Run parser/runtime property tests

Examples:
  scripts/run-test-layer.sh contract
  scripts/run-test-layer.sh contract-build
  scripts/run-test-layer.sh subsystem dom_navigation_dialog
  scripts/run-test-layer.sh integration issue_174_175
  BROWSER_TESTER_PROPTEST_CASES=64 \
  BROWSER_TESTER_RUNTIME_PROPTEST_CASES=64 \
  scripts/run-test-layer.sh fuzz
EOF
}

if [[ $# -lt 1 ]]; then
  usage
  exit 1
fi

layer="$1"
shift

case "$layer" in
  contract)
    exec cargo test --test contract_harness_core "$@"
    ;;
  contract-build)
    exec cargo test --no-run --test contract_harness_core "$@"
    ;;
  subsystem)
    exec cargo test --lib "$@"
    ;;
  integration)
    exec cargo test --test integration_suite "$@"
    ;;
  fuzz)
    exec cargo test --test parser_property_fuzz_test --test runtime_property_fuzz_test "$@"
    ;;
  -h|--help|help)
    usage
    ;;
  *)
    echo "Unknown layer: $layer" >&2
    echo >&2
    usage >&2
    exit 1
    ;;
esac

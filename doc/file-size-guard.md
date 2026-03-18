# File Size Guard

This repository uses a lightweight file-size guard to keep large Rust files visible.

## Thresholds

- review threshold: `800` lines
- hard threshold: `1500` lines

The current policy from `next-action.md` is:

- over `800` lines means "split should be considered"
- over `1500` lines means "there should be an explicit split plan"

## Scope

The guard currently treats files differently by role.

Production Rust:

- scans `src/**/*.rs`
- excludes `src/tests/**`
- warns above `800`
- fails above `1500` unless the file is listed in `doc/file-size-guard-allowlist.txt`

Test-suite Rust:

- scans `src/tests/**/*.rs` and `tests/**/*.rs`
- reports files above `800`
- does not fail yet

This softer treatment for tests is intentional because the repository still has several large historical regression suites that have not been split yet.

## Commands

Run the guard directly:

```bash
scripts/check-file-size-guard.sh
```

Override thresholds:

```bash
BROWSER_TESTER_FILE_WARN_LINES=900 \
BROWSER_TESTER_FILE_FAIL_LINES=1800 \
scripts/check-file-size-guard.sh
```

## Allowlist Rules

The allowlist lives in `doc/file-size-guard-allowlist.txt`.

Rules:

- only production Rust files above the hard threshold belong there
- removing a file from the allowlist is part of finishing the split
- stale allowlist entries fail the guard so the list stays current

## Maintenance Intent

This guard is not meant to block all large files immediately.
It is meant to prevent new oversized production files from appearing silently while the remaining hotspots are being split down over time.

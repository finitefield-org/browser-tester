# HTML Spec Conformance TODO

## Status

- `P0` through `P4` are complete.
- Rolling maintenance work is complete for the current pass.
- Latest full verification: `cargo test --lib` with `2561 passed, 0 failed`.
- No new test-only mock is currently required.

## Current Posture

- The backlog is dormant/on-demand.
- Reopen work only if one of these happens:
  - a new public API family is exposed
  - a browser-comparison-backed regression cluster appears
  - a harness/modeling change broadens a stabilized contract

## Next Task

- [ ] `Maintenance: Trigger-driven selective intake reopening`
  - reopen the smallest justified selective intake slice only when a concrete trigger appears
  - otherwise keep the roadmap dormant

# ADR 0001: Keep the Rewrite Split by Subsystem from Day One

## Status

Accepted

## Context

The current `browser-tester` crate grew from a compact harness into a much broader browser-like runtime.
`next.md` identifies one recurring problem: public `Harness` ergonomics stayed good, but internal responsibilities became too concentrated and too hard to police.

## Decision

The rewrite under this workspace starts as a workspace with four crates:

- `browser-tester` for the public facade
- `bt-dom` for DOM and selector ownership
- `bt-runtime` for session state, scheduler, services, and mocks
- `bt-script` for script parsing and evaluation

## Consequences

Positive:

- subsystem ownership is visible in code layout
- internal boundaries are harder to violate accidentally
- documentation can point to concrete ownership from the start

Negative:

- more crates mean a little more boilerplate in Phase 0
- cross-crate refactors require deliberate coordination

This tradeoff is intentional because the rewrite values maintainable boundaries over fast short-term expansion.

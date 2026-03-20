# ADR 0001: Clean-Room Workspace and Thin Facade

## Status

Accepted.

## Context

The Zig rewrite is starting from phase 0, so the first decision is not which browser feature to implement.
It is how to keep the public API small while the internal runtime grows in a controlled way.

The Rust rewrite in `../next/` shows that a `Harness`-centered API is the right starting point, but it also shows that the facade can become too large if ownership is not split early.

## Decision

- keep `Harness` as a thin public facade
- keep `Session` internal
- reserve mock and debug state inside `Session`
- document subsystem ownership before adding feature code
- treat deterministic mocks as first-class APIs when they become public

## Consequences

- the public surface starts small and stays reviewable
- capability decisions are made before implementation work starts
- the workspace can grow phase by phase without collapsing into a single large module


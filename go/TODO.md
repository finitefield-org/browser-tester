# Go Workspace Backlog

This file tracks the remaining work for the `go/` rewrite workspace.
The current codebase builds and `go test ./...` passes; the remaining items are bounded expansion, hardening, and documentation upkeep.

## P0 - Keep The Current Surface Stable

- [ ] Keep `go test ./...` and the release checklist green before merging any new slice.
- [ ] Keep unsupported behavior explicit. If a slice is incomplete, return a typed unsupported error instead of silently falling back.
- [ ] Keep the public facade thin. Do not add new `Harness` setters when a debug view or mock family is a better fit.
- [ ] Keep public API changes synchronized with `README.md`, `go/doc/capability-matrix.md`, `go/doc/subsystem-map.md`, and `go/doc/mock-guide.md`.

## P1 - Hardening

- [ ] Add or refresh internal subsystem tests for any change under `internal/dom`, `internal/runtime`, `internal/script`, or `internal/mocks`.
- [ ] Add or refresh public contract tests for any new or changed facade behavior.
- [ ] Add regression tests for every bug fix that changes observable behavior.
- [ ] Keep fuzz and property coverage current for parser, selector, scheduler, location/history, cookie/window.name, and mock-registry boundaries.

## P2 - Selector And Query Expansion

- [ ] Add new selector slices only when a user-visible gap appears and the HTML standard calls for it.
- [ ] Keep document-level and element-level query semantics aligned with the shared selector engine.
- [ ] Add additional live collection slices only when a concrete gap justifies the cost.
- [ ] Extend beyond the current bounded pseudo-class slice only when a specific scenario requires it.

## P2 - Reflection, Mutation, And Serialization

- [ ] Fill any missing bounded reflection helpers, `classList` / `dataset` behavior, or tree-mutation slices only when tests expose a gap.
- [ ] Keep `textarea` reset-default synchronization covered by tests.
- [ ] Keep `WriteHTML()` rollback behavior covered by tests.
- [ ] Verify live collections remain coherent after mutation.

## P3 - Mock Families And Debug Surfaces

- [ ] Add new mock families only through the runtime registry and expose them through the typed facade.
- [ ] Document seed state, capture behavior, failure injection, reset semantics, and a minimal example for every new mock family.
- [ ] Add new debug snapshots only when they explain a real regression or user-visible gap.
- [ ] Avoid growing `Harness` into a bag of `set_*` methods.

## Explicit Non-Goals

- Renderer and layout engine.
- External network I/O.
- Legacy or deprecated spec branches unless the capability matrix explicitly adds them.

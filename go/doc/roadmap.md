# Roadmap

This is the recommended build order for the Go workspace.
The phases are intentionally sequential at the start so the public facade stays small and the implementation stays testable.

## Phase 0: Scaffold

- Create the module, package root, internal packages, and CI-friendly tests.
- Define the public facade, error taxonomy, and explicit builder config.
- Land `Subsystem Map`, `Capability Matrix`, `Implementation Guide`, and `Mock Guide` before adding behavior.
- The scaffold is present now; later phases should extend it behind the thin facade rather than widening the API early.

Exit criteria:

- the package compiles
- the facade is thin
- the docs are in place

## Phase 1: DOM Core

- Parse HTML into the internal DOM store.
- Implement the initial selector subset.
- Implement DOM dump and assertion helpers.

Exit criteria:

- HTML round-trips deterministically in tests
- selectors work for the first supported slice

## Phase 2: Script Core

- Implement the minimum script parser/evaluator slice.
- Add host bindings needed for inline bootstrap.
- Keep unsupported syntax explicit.

Exit criteria:

- inline scripts can mutate the DOM
- errors are classified and repeatable

## Phase 3: Events and Form Controls

- Add event dispatch, bubbling, default actions, and user-like actions.
- Add input, checkbox, select, focus, blur, and submit behavior.

Exit criteria:

- deterministic event order
- deterministic form-control state updates

## Phase 4: Deterministic Runtime and Mocks

- Add fake clock, scheduler, timers, and microtasks.
- Add typed mock families and thin `Harness` actions.
- Keep capture and failure injection explicit.

Exit criteria:

- mock families are inspectable
- time-based behavior is deterministic

## Phase 5: Hardening

- Add subsystem tests, public contract tests, regression tests, and property tests.
- Define the release checklist.

Exit criteria:

- behavior is covered at the public boundary and the internal boundary

## Phase 6: Selector and Query Expansion

- Expand selector support in bounded slices.
- Add script-side query APIs that reuse the same selector engine.
- Add live collections only as needed by user-visible gaps.

Exit criteria:

- DOM and script querying use the same core selector logic

## Phase 7: Reflection, Mutation, Serialization

- Add attribute reflection, class and dataset views, tree mutation primitives, and HTML serialization/insertion surfaces.
- Keep each slice bounded and documented.

Exit criteria:

- mutation updates the DOM deterministically
- live collections stay coherent after mutation

## Working Rules

- Do not move to a later phase until the earlier phase is covered by tests.
- Do not add a new public `Harness` method until the capability matrix has a row for it.
- Use `../html-standard/` when adding or changing HTML behavior.
- Prefer small slices over large parity pushes.

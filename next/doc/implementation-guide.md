# Implementation Guide

This document explains how to actually build out the `next/` rewrite from its current Phase 1 baseline.

Use it together with:

- [architecture.md](architecture.md) for the target shape
- [subsystem-map.md](subsystem-map.md) for ownership decisions
- [capability-matrix.md](capability-matrix.md) for support-level decisions
- [roadmap.md](roadmap.md) for the staged milestone plan

## Core Rule

Do not grow `next/` by adding scattered features opportunistically.
Each change should add one capability through a narrow vertical slice:

1. decide the owning subsystem
2. define the contract with tests
3. implement inside the subsystem
4. connect through `Session`
5. expose through `Harness` only if it belongs in the public API
6. update docs when the capability becomes public

## Recommended Build Order

The safest way to turn the Phase 0 skeleton into a usable runtime is:

1. DOM bootstrap
2. selector subset
3. read-only assertions
4. script runtime minimum slice
5. event dispatch
6. form controls and default actions
7. deterministic mocks wired into runtime behavior
8. hardening and publication work

This order keeps the public facade thin and avoids implementing user actions before the DOM and selector layers are trustworthy.

Slices 1 through 3 are already implemented in this workspace; use the same vertical-slice pattern for later phases.

## First Vertical Slices

### Slice 1: Minimal HTML Tree Builder

Goal:

- `Harness::from_html(...)` should create a real document tree, not just store source text

Primary owner:

- `bt-dom`

Suggested scope:

- document node
- element nodes
- text nodes
- parent/child relationships
- basic attribute parsing

Tests to add first:

- `DomStore` builds a document with one child element
- nested elements preserve order
- text nodes are attached correctly
- malformed input returns an explicit parse error

Do not add yet:

- script execution
- event behavior
- form semantics

### Slice 2: Minimal Selector Subset

Goal:

- support `#id`, tag, and `[attr]` selection

Primary owner:

- `bt-dom`

Suggested scope:

- selector parsing for the guaranteed Phase 1 subset only
- explicit errors for unsupported selectors
- index-backed lookup where appropriate

Tests to add first:

- `#id` finds the expected node
- tag selectors return multiple nodes in document order
- `[attr]` matches presence
- unsupported selector syntax fails explicitly

Public outcome:

- `Harness::assert_exists(...)` can become the first real DOM-facing assertion

### Slice 3: Read-Only Debug and Assertion Layer

Goal:

- make the DOM inspectable before mutating it through user actions

Primary owner:

- `bt-dom` for the data
- `browser-tester` for the public assertion surface

Suggested scope:

- `assert_exists`
- `dump_dom`
- small DOM snippet support for assertion errors

Tests to add first:

- successful existence checks
- missing selector produces useful assertion text
- DOM dump is stable for simple fixtures

### Slice 4: Script Minimum Slice

Goal:

- make one narrow inline-script scenario work end to end

Primary owner:

- `bt-script`

Recommended first supported scenario:

- `document.getElementById("out").textContent = "Hello";`

Why this slice:

- it proves the binding seam
- it exercises DOM lookup and mutation
- it avoids jumping directly to broad JavaScript compatibility

Tests to add first:

- inline script mutates text content
- missing element access returns a script-side error
- unsupported syntax still fails explicitly

### Slice 5: Event Target Dispatch

Goal:

- register a listener and dispatch a simple event to the target node

Primary owner:

- `bt-runtime` for event orchestration
- `bt-script` for callback bridging

Suggested scope:

- listener registration
- target-phase dispatch
- deterministic callback order

Delay until later:

- full capture/bubble behavior
- complex default actions

### Slice 6: Forms and User Actions

Goal:

- support the first realistic form test through `Harness`

Recommended order:

1. input value state
2. `type_text`
3. checkbox state
4. `set_checked`
5. button click default behavior
6. submit behavior

Primary owners:

- `bt-dom` for control state
- `bt-runtime` for default-action orchestration
- `browser-tester` for thin public entry points

### Slice 7: Mock Integration

Goal:

- connect the typed mock registry to real runtime behavior

Recommended order:

1. dialogs
2. clipboard
3. fetch
4. location
5. file input
6. download capture

Why this order:

- dialogs and clipboard are simpler than fetch/navigation
- fetch and location tend to widen the service surface quickly

## Decision Flow for Each Change

Before implementing anything, answer these questions:

1. Which crate owns the long-lived state?
2. Is this a stable public API, a test-only mock, a debug helper, or internal-only machinery?
3. Can it be expressed as a smaller capability slice?
4. What is the explicit failure behavior for unsupported input?
5. Which test layer should lock this in?

If the answers are unclear, the change is probably too wide.

## Test Strategy by Stage

### Early Phases

Prefer:

- subsystem tests
- small public contract tests
- explicit failure-path tests

Avoid:

- large issue-driven regression fixtures before a capability stabilizes
- browser-comparison work before the public contract is defined

### Once a Capability Is Public

Require:

- a public contract test
- a subsystem test in the owning crate
- at least one failure-path test
- docs that match the actual API

### For New Test-Only Mocks

Require:

- response injection or seed-state coverage
- failure-path coverage
- call capture or artifact capture coverage
- README update
- `doc/mock-guide.md` update

## Update Rules

Update these files when the corresponding decisions change:

- `doc/capability-matrix.md`: support level or public guarantee changed
- `doc/mock-guide.md`: a new public mock or new capture mode exists
- `doc/subsystem-map.md`: ownership guidance changed
- `README.md`: user-visible public entry point changed

## Practical Working Loop

Use this loop for daily implementation work:

1. pick one slice from the current phase
2. decide the owning subsystem
3. write or extend tests
4. implement the subsystem behavior
5. wire through `Session`
6. expose through `Harness` only if needed
7. update docs
8. run `cargo test`

## Recommended Immediate Next Task

The next highest-value task is:

1. implement minimal HTML tree building in `bt-dom`
2. implement `#id` selector support
3. enable `Harness::assert_exists(...)`

That is the first slice that turns the Phase 0 skeleton into a real, usable contract.

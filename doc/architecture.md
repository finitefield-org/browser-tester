# browser-tester Architecture

## Overview

`browser-tester` is a deterministic browser-like runtime for Rust tests.
It is designed for cases where launching a real browser is too heavy, too slow, or too hard to control precisely.

The crate runs HTML, DOM interaction, and script behavior inside a single Rust process and exposes a compact `Harness` API for tests.

The design goal is not full browser compatibility.
The goal is to provide a stable, deterministic subset that is useful for browser-style testing.

## Goals

- Execute HTML, DOM, and script tests within a single process.
- Avoid depending on an external browser, WebDriver, or Node.js.
- Keep time, randomness, navigation, and browser-like APIs deterministic.
- Make test-only mocks first-class features instead of ad hoc stubs.
- Keep the public testing surface centered on `Harness`.

## Non-Goals

- Full browser compatibility
- Real rendering or layout
- General-purpose network behavior
- Full iframe or multi-process browsing semantics
- Exhaustive implementation of every Web API

## Public Entry Point

The main public API is the `Harness` type.

Typical flow:

1. Build a harness from HTML.
2. Perform user-like actions by selector.
3. Assert DOM state or inspect deterministic artifacts.

Main public categories:

- constructors
- user actions
- assertions
- timer and scheduler controls
- mocks
- trace and debug helpers

The current public support levels are tracked separately in
[capability-matrix.md](capability-matrix.md).

## Current Crate Structure

The project is currently implemented as a single crate with internal module layering.

Top-level modules:

- `src/lib.rs`
- `src/harness_api.rs`
- `src/core_dom_utils.rs`
- `src/runtime_state.rs`
- `src/runtime_values.rs`
- `src/script_ast.rs`
- `src/selector.rs`
- `src/core_impl/`

High-level internal areas:

- `src/core_impl/dom`
- `src/core_impl/parser`
- `src/core_impl/runtime`
- `src/core_impl/intl`

This is still a single-crate architecture, but subsystem boundaries are visible in the module tree.

## Runtime Lifecycle

At a high level, a harness session works like this:

1. Parse input HTML.
2. Build the internal DOM.
3. Initialize browser-like globals.
4. Register and execute inline scripts in document order.
5. Let tests drive the DOM through `Harness`.
6. Dispatch events, run default actions, and drain microtasks or timers as needed.

Important implications:

- script execution order is deterministic
- timers never wait on wall-clock time
- mock-backed APIs stay under Rust-side control

## DOM Model

The crate uses an in-memory DOM model managed in Rust.

Key characteristics:

- arena-style node storage
- stable internal node identifiers
- DOM mutation handled in-repo
- selector resolution handled in-repo

The DOM layer is responsible for:

- tree structure
- tag and attribute access
- form-control state
- mutation helpers
- serialization helpers used by assertions and debugging

This design keeps DOM behavior under crate control, which is useful for deterministic tests but increases maintenance cost as the surface grows.

## Selectors

Selectors are implemented inside the crate rather than delegated to an external browser engine.

Supported behavior includes:

- id, class, tag, and attribute selectors
- combinators such as descendant and child
- a range of pseudo-class matching used by tests

Unsupported selectors are expected to fail explicitly rather than degrade silently.

This is important because silent partial selector support is hard to debug in tests.

## Script Runtime

The script layer is also self-implemented.

Main pieces:

- lexer
- parser
- AST definitions in `src/script_ast.rs`
- evaluator and runtime value model
- host bindings for DOM and browser-like APIs

This gives the project strong control over determinism and exposed behavior, but it is also one of the biggest maintenance hotspots.

In practice, the script runtime has to deal with:

- lexical environments
- closures
- callback invocation
- promise and microtask behavior
- built-in object semantics
- host-object integration

That is why script-runtime changes often have wide effects.

## Event System

The event system is browser-like but deterministic.

Core responsibilities:

- capture phase
- target phase
- bubble phase
- `preventDefault`
- `stopPropagation`
- `stopImmediatePropagation`
- default-action handling for user-like events

Representative default-action paths include:

- checkbox and radio activation
- form submission
- anchor navigation
- clipboard-related user actions
- file input activation

Tests use these paths through `Harness` methods such as `click`, `submit`, `copy`, and `paste`.

## Determinism Model

Determinism is a core design property.

The crate currently provides:

- fake clock for `Date.now()` and `performance.now()`
- deterministic timer queue
- explicit time advancement via `advance_time` and `flush`
- deterministic `Math.random()`
- mock-controlled browser-like APIs

This allows tests to control execution order precisely, especially around:

- timeouts and intervals
- microtask completion
- navigation-like transitions
- clipboard or fetch side effects

## Browser-Like Services and Mocks

The crate exposes deterministic, test-oriented behavior for several browser-like surfaces.

Important families:

- `fetch`
- `location` and history-backed mock pages
- dialogs such as `confirm` and `prompt`
- clipboard APIs
- localStorage seed state
- downloads and object URLs
- file inputs
- `matchMedia`

These are not incidental helpers.
They are a core part of the crate's value as a test runtime.

Detailed examples live in [mock-guide.md](mock-guide.md).

## Errors and Debugging

The runtime uses crate-defined error types rather than delegating diagnostics to an external browser.

Important error shapes include:

- HTML parse errors
- script parse errors
- script runtime errors
- selector errors
- assertion failures with DOM snippets

Debugging support includes:

- event and timer tracing
- log capture through `take_trace_logs`
- DOM dumping through `dump_dom`
- deterministic artifact inspection such as fetch calls, clipboard writes, and downloads

## Test Strategy

The repository currently relies on multiple layers of testing:

- minimal public contract suite under `tests/contract_harness_core.rs`
- unit-style DOM and runtime tests under `src/tests`
- integration cases under `tests/integration_cases`
- grouped integration entrypoint under `tests/integration_suite.rs`
- property and fuzz tests for parser and runtime behavior

See `doc/test-taxonomy.md` for the intended placement rules and role boundaries.
See `doc/file-size-guard.md` for the current oversized-file guardrail and exception policy.
See `doc/public-api-checklist.md` for the required steps when the public surface changes.
See `doc/subsystem-map.md` for the current ownership-oriented placement guide.

This provides strong coverage, but it also means test role separation needs active maintenance.

The most important distinction going forward is:

- public contract tests
- subsystem tests
- regression tests
- property and fuzz tests

Without that distinction, test volume alone can make maintenance harder.

## Current Maintenance Hotspots

Some files have grown very large and are natural candidates for responsibility-based splitting.

Notable examples:

- `src/core_impl/runtime/runtime_exec/member_calls_ops/value_object_helpers.rs`
- `src/core_impl/runtime/runtime_platform/script_runtime/callable_execution.rs`
- `src/core_impl/runtime/runtime_platform/script_runtime/statement_execution.rs`
- `src/core_impl/runtime/runtime_platform/dom_actions/user_actions_forms.rs`
- `src/script_ast.rs`

These hotspots matter because the current architecture is still centered on a single public facade, but the internal implementation is broad.

## Why This Document Exists

The README is now intentionally short and user-facing.
This file exists to keep architecture notes separate from:

- quick-start usage
- mock examples
- release-facing guidance

That separation lowers the maintenance cost of both the public README and the internal design record.

## Related Documents

- Capability classification: [capability-matrix.md](capability-matrix.md)
- Mock APIs and examples: [mock-guide.md](mock-guide.md)
- HTML conformance roadmap: [html-spec-conformance-roadmap.md](html-spec-conformance-roadmap.md)
- WPT audit inventory: [p3-wpt-audit-inventory.md](p3-wpt-audit-inventory.md)
- Public package README: [../README.md](../README.md)

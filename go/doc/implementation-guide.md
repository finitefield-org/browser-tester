# Implementation Guide

This guide describes how the Go workspace should be built.
It is deliberately conservative: explicit subsystems, thin public facade, and deterministic mocks first.

## Design Rules

1. Keep `Harness` thin. Its methods should delegate into runtime or mock registry objects.
2. Keep mutable state in the owning subsystem. Do not scatter the same state across facade, runtime, and DOM.
3. Keep public and test-only APIs separate. If something exists only for tests, it belongs in a mock family or a debug view.
4. Keep builder configuration explicit. Do not encode mock seeds into unrelated state.
5. Use `../html-standard/` as the reference for any HTML/DOM slice before coding it.
6. Prefer bounded slices over broad partial compatibility. Each new slice should have a clear exit criterion.
7. Do not spend implementation budget on legacy or deprecated spec behavior. Treat those branches as out of scope unless a specific, documented user-visible need requires them.

## Suggested File Layout

The exact file names can change, but the intended ownership is:

```text
browsertester/
  harness.go
  errors.go
  debug.go
  mocks.go
internal/
  dom/
    store.go
    parser.go
    selector.go
    collections.go
    serialize.go
  runtime/
    session.go
    scheduler.go
    events.go
    history.go
    location.go
  script/
    runtime.go
    parser.go
    evaluator.go
    bindings.go
  mocks/
    fetch.go
    dialogs.go
    clipboard.go
    location.go
    open.go
    close.go
    print.go
    scroll.go
    matchmedia.go
    downloads.go
    fileinput.go
    storage.go
```

## Build Order

### Phase 0: Scaffold

- Create the module, public package, internal packages, and a minimal build.
- Define the error taxonomy and the public facade types.
- Define `SessionConfig` with explicit fields.
- Define `MockRegistryView` and `DebugView` early so later APIs have a place to live.
- The current scaffold already compiles with `go test ./...`; later phases should extend it without widening the facade prematurely.

Exit criteria:

- `go test ./...` passes with skeleton tests.

### Phase 1: DOM

- Implement HTML parsing into a tree store.
- Implement selector matching for the first bounded slice.
- Implement DOM dump helpers and the initial assertion helpers. The current Go workspace already has the initial assertion slice; later work should keep it bounded rather than widening the facade.

Exit criteria:

- Parsed HTML round-trips through the DOM dump in tests.
- Selector behavior is covered by contract and regression tests.

### Phase 2: Script Runtime Minimum

- Implement the lexer/parser/evaluator slice needed for inline bootstrap.
- Add host bindings for the initial DOM and document/window accessors.
- Keep the runtime deterministic and explicit about unsupported syntax.

Exit criteria:

- Inline scripts can mutate the DOM during bootstrap.
- Missing features fail explicitly, not silently.

### Phase 3: Events and User-Like Actions

- Implement target-phase listener dispatch, default actions, and form-control state updates.
- Add `Click`, `TypeText`, `SetChecked`, `SetSelectValue`, `Focus`, `Blur`, `Dispatch`, `DispatchKeyboard`, and `Submit`.

Exit criteria:

- Target listeners and default actions are deterministic.
- Bubbling/capture remain later unless explicitly added to the matrix.
- Default actions and form updates are covered by tests.

### Phase 4: Runtime Services and Mock Registry

- Implement the deterministic clock, timers, microtasks, and scheduler.
- Implement typed mock families for fetch, dialogs, clipboard, location, open, close, print, scroll, matchMedia, downloads, file input, and storage.
- Add public mock actions on `Harness` as thin wrappers.

Exit criteria:

- Every family supports seed state, capture, and reset.
- Public actions remain thin and do not duplicate registry logic.

### Phase 5: Hardening

- Add subsystem tests for internal packages.
- Add public contract tests for the facade.
- Add regression tests for issue reproductions.
- Add fuzz/property tests for parser and scheduler boundaries.

Exit criteria:

- The implementation can be guarded by a repeatable publication checklist.

### Phase 6: Selector and Query Expansion

- Expand selectors in bounded slices.
- Add script-side `querySelector`, `querySelectorAll`, `matches`, and `closest`.
- Add live collection slices only when a user-visible gap demands them.

Exit criteria:

- Query APIs reuse the same selector engine as the DOM layer.

### Phase 7: Reflection, Mutation, and Serialization

- Add attribute reflection, class/dataset views, selector-based tree mutation helpers, and HTML serialization/insertion helpers.
- Keep the supported slice bounded and documented.

Exit criteria:

- Mutation updates selectors and live collections deterministically.

## Test Policy

- Use `*_contract_test.go` for public facade behavior.
- Use `*_test.go` under `internal/...` for subsystem behavior.
- Use `*_regression_test.go` for issue reproductions.
- Use fuzz tests for parser, selector, and scheduler boundaries.
- Keep tests close to the behavior they protect.

## Change Rules

- If a new behavior belongs only to tests, add it as a mock family or a debug view.
- If a new behavior is user-facing, update the capability matrix and `README.md` before or with the code.
- If a new mock family is added, update the mock guide and add a minimal example plus failure coverage.
- Do not let `Harness` become a bag of setter methods.
- Legacy and deprecated spec paths are not implementation targets unless the capability matrix explicitly lists them for a concrete compatibility reason.

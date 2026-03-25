# Subsystem Map

Use this document before adding code so ownership stays explicit.

## Public Facade

Owns:

- `Harness`
- `HarnessBuilder`
- `Error` and its classification helpers
- `DebugView`
- `MockRegistryView`
- public live view and snapshot helper types such as `ClassListView`, `DatasetView`, `OptionLabel`, `OptionValue`, and `OptgroupLabel`
- public tree-mutation helpers such as `TextContent`, `SetTextContent`, `ReplaceChildren`, and `CloneNode`
- public seed/value helper types that are part of constructors or mock APIs

Location:

- `browsertester` package at the module root

Choose this layer when the question is:

- is this really part of the public API?
- should this stay a thin facade or move into a subsystem?
- should this be read-only inspection, user-like action, or test-only mock access?

## DOM

Owns:

- node identifiers
- DOM tree storage
- HTML parsing
- selector matching
- live-collection indexes and side tables
- DOM serialization helpers
- reflected tree state that is owned by the DOM store

Location:

- `internal/dom`

Choose this layer when the question is:

- what nodes exist and how are they related?
- how should a DOM mutation update indexes or side tables?
- how should an HTML fragment serialize or round-trip?

## Runtime

Owns:

- `Session`
- scheduler and fake time
- deterministic browser-like services
- navigation and history state
- cookie jar, web storage, and scroll state
- event routing and dispatch
- test-only mock implementations
- trace and debug state

Location:

- `internal/runtime`

Choose this layer when the question is:

- when should a callback run?
- how should a mock capture data?
- where should shared browser-like session state live?
- how should a bootstrap action affect the current session?

## Script

Owns:

- script lexer
- parser
- evaluator
- host bindings
- microtask execution hooks tied to script runtime semantics

Location:

- `internal/script`

Choose this layer when the question is:

- how should this source text parse?
- how should a script expression evaluate?
- how does a host object bridge into script?

## Mocks

Owns:

- family structs for fetch, dialogs, clipboard, location, open, close, print, scroll, matchMedia, downloads, file input, and storage
- capture logs
- failure injection
- reset behavior

Location:

- `internal/mocks`

Choose this layer when the question is:

- what should be captured?
- what should be seeded?
- what should fail deterministically?

## Placement Rules

1. Put long-lived state in the subsystem that owns that state.
2. Keep `Harness` entry points thin and delegating.
3. Do not let script-runtime types leak into DOM or runtime data models.
4. Add a new public API only after deciding whether it belongs on `Harness`, `DebugView`, or a mock family.
5. Add a new mock in `internal/runtime` or `internal/mocks`, then wire it through the public facade without bypassing the registry.
6. Use explicit config fields for builder seeds and bootstrap failures. Do not hide them in storage or other unrelated state.
7. Check `../html-standard/` before implementing any new HTML or DOM slice.

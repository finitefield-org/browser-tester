# Subsystem Map

Use this document before adding code so ownership stays explicit.

## Public Facade

Owns:

- `Harness`
- `HarnessBuilder`
- read-only inspection methods
- user-facing actions
- deterministic clock helpers
- typed mock registry access
- `Error`
- `Result(T)`
- `StorageSeed`
- `MockRegistry`

Location:

- `src/root.zig`
- `src/harness.zig`
- `src/errors.zig`

Choose this layer when the question is:

- is this really part of the public API?
- should this stay a thin facade or move into a subsystem?

## Session

Owns:

- copied configuration state
- internal DOM store
- internal script runtime state
- event listener registry
- focus/target selector-state snapshots
- fake clock state
- mock registry
- future long-lived runtime state

Location:

- `src/session.zig`
- `src/mocks.zig`

Choose this layer when the question is:

- where should owned rewrite state live?
- what does the session snapshot contain?

## DOM

Owns:

- node identifiers
- DOM tree storage
- HTML parsing
- DOM dump helpers
- selector matching
- DOM indexes and side tables

Location:

- `src/dom.zig`

Choose this layer when the question is:

- what nodes exist and how are they related?
- how should a DOM mutation update indexes or side tables?

## Runtime

Owns:

- scheduler and fake time
- deterministic browser-like services
- test-only mock implementations
- trace and debug state

Location:

- `src/session.zig`
- `src/mocks.zig`

Choose this layer when the question is:

- when should a callback run?
- how should a mock capture data?
- where should shared browser-like session state live?

## Script

Owns:

- script lexer
- parser
- evaluator
- host bindings

Location:

- `src/script.zig`

Choose this layer when the question is:

- how should this source text parse?
- how should a script expression evaluate?
- how does a host object bridge into script?

## Placement Rules

1. Put long-lived state in the subsystem that owns that state.
2. Keep `Harness` entry points thin and delegating.
3. Do not let script-runtime types leak into DOM or runtime data models.
4. Add a new public API only after deciding whether it belongs on `Harness`, a debug view, or a mock family.
5. Add a new mock in runtime, then wire it through the public facade without bypassing the registry.

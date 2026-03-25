# Subsystem Map

This map is the placement guide for this rewrite.
Use it before adding code so ownership stays explicit.

## Public Facade

Owns:

- `Harness`
- `HarnessBuilder`
- public error taxonomy
- thin public views such as `MockRegistryView` and `DebugView`

Location:

- `crates/browser-tester/`

Choose this layer when the question is:

- "is this really part of the public API?"
- "should this stay a thin facade or move into a subsystem?"

## DOM

Owns:

- node identifiers
- DOM tree storage
- HTML parsing
- selector matching
- DOM indexes and side tables

Location:

- `crates/bt-dom/`

Choose this layer when the question is:

- "what nodes exist and how are they related?"
- "how should a DOM mutation update indexes or side tables?"

## Runtime

Owns:

- `Session`
- scheduler and fake time
- deterministic browser-like services
- test-only mock implementations
- trace and debug state

Location:

- `crates/bt-runtime/`

Choose this layer when the question is:

- "when should a callback run?"
- "how should a mock capture data?"
- "where should shared browser-like session state live?"

## Script

Owns:

- script lexer
- parser
- evaluator
- host bindings
- microtask execution hooks tied to script runtime semantics

Location:

- `crates/bt-script/`

Choose this layer when the question is:

- "how should this source text parse?"
- "how should a script expression evaluate?"
- "how does a host object bridge into script?"

## Placement Rules

1. Put long-lived state in the subsystem that owns that state.
2. Keep `Harness` entry points thin and delegating.
3. Do not let script-runtime types leak into DOM or runtime data models.
4. Add a new public API only after deciding whether it belongs on `Harness`, a debug view, or a mock family.
5. Add a new mock in `bt-runtime`, then wire it through the public facade without bypassing the registry.

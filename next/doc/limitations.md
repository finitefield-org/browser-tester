# Limitations

The `next/` workspace is intentionally at Phase 0.

What exists today:

- a compilable crate graph
- public and internal type skeletons
- deterministic session bootstrap
- explicit mock families and debug hooks

What does not exist yet:

- HTML parsing
- selector matching
- script evaluation semantics
- inline script bootstrapping
- event propagation
- default actions
- form-control behavior
- realistic timer and microtask execution

## Important Consequence

Public methods such as `click`, `type_text`, `focus`, `dispatch`, and assertion helpers are present only as explicit placeholders.
They return a clear "planned for later phase" error instead of pretending to work partially.

## Still Out of Scope

Even after later phases, this rewrite does not aim to provide:

- real rendering
- CSS layout
- arbitrary browser compatibility
- real network access
- service worker support
- exhaustive Web API coverage


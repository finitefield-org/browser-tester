# Limitations

The `next/` workspace has reached Phase 2 for DOM and script core.

What exists today:

- a compilable crate graph
- public and internal type skeletons
- deterministic session bootstrap
- real HTML parsing and DOM tree construction for the Phase 1 subset
- selector matching for `#id`, tag, and `[attr]`
- `assert_exists` and DOM dump helpers
- inline `<script>` bootstrapping with a minimal script parser/evaluator
- listener registration through the script host seam
- explicit mock families and debug hooks

What does not exist yet:

- event propagation
- default actions
- form-control behavior
- realistic timer and microtask execution
- broader selector support beyond the Phase 1 subset

## Important Consequence

Public methods such as `click`, `type_text`, `focus`, `dispatch`, `assert_text`, `assert_value`, `assert_checked`, and `set_select_value` are present only as explicit placeholders.
They return a clear "planned for later phase" error instead of pretending to work partially.

## Still Out of Scope

Even after later phases, this rewrite does not aim to provide:

- real rendering
- CSS layout
- arbitrary browser compatibility
- real network access
- service worker support
- exhaustive Web API coverage

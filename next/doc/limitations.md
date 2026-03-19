# Limitations

The `next/` workspace has reached Phase 3 for DOM, script, events, and forms.

What exists today:

- a compilable crate graph
- public and internal type skeletons
- deterministic session bootstrap
- real HTML parsing and DOM tree construction for the Phase 1 subset
- selector matching for `#id`, tag, and `[attr]`
- `assert_exists` and DOM dump helpers
- inline `<script>` bootstrapping with a minimal script parser/evaluator
- listener registration through the script host seam
- target-phase event dispatch and form-control state updates
- `click`, `type_text`, `set_checked`, `submit`, `dispatch`, `assert_value`, and `assert_checked`
- explicit mock families and debug hooks

What does not exist yet:

- event propagation beyond target phase
- cancelable default actions and richer event lifecycle semantics
- realistic timer and microtask execution
- broader selector support beyond the Phase 1 subset

## Important Consequence

Public methods such as `focus`, `blur`, and `set_select_value` are still explicit placeholders.
They return a clear "planned for a later phase" error instead of pretending to work partially.

## Still Out of Scope

Even after later phases, this rewrite does not aim to provide:

- real rendering
- CSS layout
- arbitrary browser compatibility
- real network access
- service worker support
- exhaustive Web API coverage

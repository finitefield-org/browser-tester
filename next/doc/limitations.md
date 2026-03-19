# Limitations

The `next/` workspace has reached Phase 4 for DOM, script, events, forms, fake time, and deterministic mock wiring, and Phase 5 for hardening and publication checks.

What exists today:

- a compilable crate graph
- public and internal type skeletons
- deterministic session bootstrap
- real HTML parsing and DOM tree construction for the Phase 1 subset
- selector matching for `#id`, tag, `[attr]`, `.class`, `tag.class`, and `#id.class`
- `assert_exists` and DOM dump helpers
- inline `<script>` bootstrapping with a minimal script parser/evaluator
- listener registration through the script host seam
- target-phase event dispatch and form-control state updates
- `click`, `type_text`, `set_checked`, `submit`, `dispatch`, `assert_value`, and `assert_checked`
- explicit mock families, debug hooks, and public mock actions for `fetch`, dialogs, clipboard, location, file input, and download capture
- contract, regression, and property test suites
- quick and hardening test profiles plus a publication checklist

What does not exist yet:

- descendant combinators and child combinators
- selector hardening beyond the bounded Phase 6 slice set

## Important Consequence

Public methods such as `focus`, `blur`, `set_select_value`, `fetch`, `confirm`, `prompt`, `read_clipboard`, `write_clipboard`, `navigate`, `set_files`, and `capture_download` are now supported and dispatch or capture through the same deterministic runtime.

## Still Out of Scope

Even after later phases, this rewrite does not aim to provide:

- real rendering
- CSS layout
- arbitrary browser compatibility
- real network access
- service worker support
- exhaustive Web API coverage

# Roadmap

This roadmap mirrors the staged plan from [`next.md`](../../next.md) and turns it into the working order for `next/`.

For day-to-day implementation sequencing inside each phase, also see [implementation-guide.md](implementation-guide.md).

## Phase 0: Skeleton

Delivered in this workspace:

- independent Rust workspace under `next/`
- `HarnessBuilder`
- `Session`
- `DomStore`
- scheduler and mock registry skeletons
- error taxonomy
- design-document set

Exit criteria:

- build an empty session without HTML
- keep URL and storage seed configuration
- compile and test the workspace

## Phase 1: DOM Core

Delivered in this workspace:

- HTML parser
- DOM tree construction
- selector subset
- `assert_exists`
- `dump_dom`

Exit criteria:

- build DOM from HTML text
- resolve `#id`, tag, and attribute selectors

## Phase 2: Script Runtime

Delivered in this workspace:

- lexer
- parser
- evaluator
- host bindings for `window`, `document`, and `Element`
- inline script execution

Exit criteria:

- simple DOM mutation through script works
- event handlers can be registered

## Phase 3: Events and Forms

Delivered in this workspace:

- event dispatch with ancestor bubbling and capture listeners
- default action registry for checkbox toggles and submit buttons
- form controls for text input, textarea, checkbox, and select state
- `click`, `type_text`, `set_checked`, `set_select_value`, `focus`, `blur`, `submit`, `dispatch`
- `assert_value` and `assert_checked`

Exit criteria:

- common form-oriented UI tests pass

## Phase 4: Determinism and Mocks

Delivered in this workspace:

- fake clock hardening
- microtask queue semantics
- fetch mock
- dialogs
- clipboard
- location mock
- file input mock

Exit criteria:

- realistic mock-heavy tests can be expressed through the public facade

## Phase 5: Hardening

Planned work:

- public contract tests
- subsystem tests
- regression suite
- property tests
- documentation polish
- publish checklist

Exit criteria:

- docs and code agree
- quick and hardening test profiles both exist

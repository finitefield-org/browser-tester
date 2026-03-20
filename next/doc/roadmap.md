# Roadmap

This roadmap mirrors the staged plan from [`next.md`](../../next.md) and turns it into the working order for `next/`.

For day-to-day implementation sequencing inside each phase, also see [implementation-guide.md](implementation-guide.md).

Status note:

- Phases 0 through 6 are complete in this workspace.
- Phase 7 is complete in this workspace; slices 1 through 4 are delivered, sibling selectors (`A + B`, `A ~ B`), selector lists (`A, B`), and a small pseudo-class slice are also delivered as backlog slices, and post-Phase-7 collection slices add `querySelectorAll` with minimal `NodeList` support plus `Element.children`, `getElementsByTagName`, `getElementsByTagNameNS`, `getElementsByClassName`, `getElementsByName`, `document.forms` / `form.elements`, `select.options`, `document.images` / `document.links`, and `document.all` collection support. Future selector work should stay backlog-driven.

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
- download capture

Exit criteria:

- realistic mock-heavy tests can be expressed through the public facade

## Phase 5: Hardening

Delivered in this workspace:

- public contract tests
- subsystem tests
- regression suite
- property tests
- documentation polish
- publish checklist
- quick and hardening test profiles

Exit criteria:

- docs and code agree
- quick and hardening test profiles both exist

## Phase 6: Selector Expansion

Delivered slices 1 through 4:

- class selectors and compound simple selectors
  - `.class`, `tag.class`, `#id.class`
  - `DomStore::select`, `assert_exists`, and action resolution now share the selector engine

- descendant combinators
  - `A B`
  - nested DOM matching and document-order behavior now work through the same selector engine

- child combinators
  - `A > B`
  - direct-child matching and false-positive avoidance now work through the same selector engine

Phase 6 complete:

- selector hardening and regression coverage are delivered

Exit criteria:

- `DomStore::select` resolves the bounded selector set deterministically
- existing `Harness` assertions and actions work with the new selector forms
- unsupported syntax still fails explicitly
- quick and hardening profiles stay green

## Phase 7: Script DOM Query Expansion

Delivered slices 1 through 4:

- `document.querySelector(...)` and `element.querySelector(...)`
  - document-order first match, subtree-scoped lookup, `null` on miss

- `Element.matches(...)`
  - current-element only, boolean return

- `Element.closest(...)`
  - self-inclusive ancestor walk, `null` on miss

Post-Phase-7 slices:

- query selector collections
  - `querySelectorAll` with minimal `NodeList` support
  - `length` and `item()` only
- element child collections
  - `Element.children` with minimal `HTMLCollection` support
  - `length`, `item()`, and `namedItem()`
- tag-name collections
  - `getElementsByTagName` with minimal `HTMLCollection` support
  - `length`, `item()`, and `namedItem()`
- namespace-aware tag-name collections
  - `getElementsByTagNameNS` with minimal `HTMLCollection` support
  - `length`, `item()`, and `namedItem()`
  - bounded to the HTML, SVG, and MathML namespace URIs plus `*`
- class-name collections
  - `getElementsByClassName` with live `HTMLCollection` support
  - `length`, `item()`, and `namedItem()`
- name collections
  - `getElementsByName` with live `NodeList` support
  - `length` and `item()`
- form collections
  - `document.forms` with live `HTMLCollection` support
  - `form.elements` with live `HTMLCollection` support
  - `length`, `item()`, and `namedItem()`
- select collections
  - `select.options` with live `HTMLCollection` support
  - `length`, `item()`, and `namedItem()`
- image and link collections
  - `document.images` with live `HTMLCollection` support
  - `document.links` with live `HTMLCollection` support
  - `length`, `item()`, and `namedItem()`
- all-elements collection
  - `document.all` with live `HTMLCollection` support
  - `length`, `item()`, and `namedItem()`
- selector lists
  - comma-separated selector lists are available through the same bounded engine
  - document-order union and deduplication
- simple pseudo-classes
  - `:first-child`, `:last-child`, `:checked`, `:disabled`, and `:enabled` are available through the same bounded engine
  - `:nth-child`, `:not`, `:is`, and broader pseudo-class parsing deferred

Exit criteria:

- inline scripts can use selector-based lookup without a new `Harness` API
- selector grammar stays bounded and deterministic
- unsupported syntax still fails explicitly
- docs, contract tests, and regression tests agree

## After Phase 7

Operating rule:

- future selector work should stay backlog-driven and narrow
- `querySelectorAll` with minimal `NodeList` support is available, `Element.children`, `getElementsByTagName`, `getElementsByTagNameNS`, `getElementsByClassName`, `getElementsByName`, `document.forms` / `form.elements`, `select.options`, `document.images`, `document.links`, and `document.all` collection support are also available, and selector lists and simple pseudo-classes are also available through the bounded engine

## After Phase 7: Rolling Capability Delivery

Operating rule:

1. pick one user-visible gap or regression cluster
2. decide the owning subsystem first
3. lock the change in with public contract, subsystem, and failure-path tests
4. implement inside the owning subsystem
5. expose through `Harness` only if the scenario cannot already be expressed
6. update public docs in the same change when the supported surface changes

Open a new named phase only when:

- multiple upcoming slices share one cross-cutting milestone
- that milestone needs its own exit criteria and user-facing status boundary

Until then, keep shipping backlog-driven capability slices under this post-Phase-7 mode.

# Roadmap

This roadmap mirrors [`next.md`](../../next.md) and turns it into the working order for the Zig rewrite.

## Phase 0: Scaffold

Delivered in this workspace:

- project skeleton
- `HarnessBuilder`
- `Harness`
- copied configuration state
- error taxonomy
- design docs

Exit criteria:

- build the owned session snapshot from captured configuration
- keep URL, HTML, and local storage seed configuration
- compile and test the workspace

## Phase 1: DOM Core

Delivered in this workspace:

- HTML parser
- DOM tree construction
- selector subset
- DOM dump helpers

Still to ship:

- read-only assertions

## Phase 2: Script Runtime

- lexer
- parser
- evaluator
- host bindings for `window`, `document`, and `Element`
- inline script execution

## Phase 3: Events and Forms

- event dispatch with bubbling and capture listeners
- default actions
- form controls
- user-facing `Harness` actions

## Phase 4: Determinism and Mocks

- fake clock hardening
- microtask semantics
- fetch, clipboard, dialog, location, and file-input mocks
- download capture

## Phase 5: Hardening

- contract tests
- regression suite
- property tests
- publication checklist

## Phase 6: Selector Expansion

- class selectors and compound simple selectors
- descendant combinators
- child combinators
- selector hardening

## Phase 7: Script DOM Query Expansion

- `document.querySelector`
- `element.querySelector`
- `Element.matches`
- `Element.closest`
- collection support
- bounded selector grammar broadening

## Phase 8: DOM Mutation and Reflection Expansion

- attribute reflection
- class and dataset views
- tree mutation primitives
- HTML serialization surfaces

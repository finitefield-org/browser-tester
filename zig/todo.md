# Zig TODO

The Rust `tests/` tree contains cases that still depend on capabilities or test hooks
that are not available in the Zig workspace yet.

## Missing modern JS syntax for the Zig port

The new `zig/tests/` cases are intentionally written against newer JS syntax and
runtime behavior. Implement these items in order, one slice at a time, so the
interpreter stays small and each step can be regression-tested.

- `let` / `var` declarations with mutable lexical bindings
  - Needed for closure-heavy flows, loop counters, and outer state updates.

- Function declarations and function expressions
  - Needed for late-bound helpers, recursive callbacks, and listener bodies.

- `return`, `if` / `else if` / `else`, and `for`
  - Needed for the control-flow-heavy callbacks in the migrated Zig tests.

- `try` / `catch` and `throw`
  - Needed for fallback-heavy helpers and error-path assertions.

- Assignment to identifiers, compound assignment, and block-scoped shadowing
  - Needed for `x = 1`, `count += 1`, and inner `index` variables that must not leak.

- Array literals, computed indexing, spread, and destructuring
  - Needed for `[...cur]`, `values[index]`, `[a, b] = [b, a]`, and callback params like `([key, value]) => ...`.

- Object literals with shorthand, spread, methods, getters, setters, and computed keys
  - Needed for `{ delimiter, ...measure }`, `get amount() { ... }`, and `obj['dark']`.

- Optional chaining
  - Needed for `actionEls.close?.addEventListener(...)`.

- Template literals and regex literals
  - Needed for `${label}|${formatted}`, HTML escaping helpers, and `split(/\r?\n/)`.

- Comparison, logical, and unary operators
  - Needed for `===`, `!==`, `&&`, `||`, `<`, `>`, `typeof`, `instanceof`, and `!value`.

- `Map`, `Object.assign`, `Object.fromEntries`, `Object.keys`, and `Array.from`
  - Needed for lookup tables, callback mapping, and key extraction.

- Array helpers used by the ported tests
  - `map`, `filter`, `find`, `findIndex`, `flat`, `flatMap`, `join`, `split`, `slice`,
    `splice`, `push`, `pop`, `includes`, `indexOf`, and `lastIndexOf`.

- `Date` constructor string parsing and `Date.prototype.toLocaleDateString`
  - Needed for date-format regressions.

- `Promise`, `async`, and `await`
  - Needed for the requestAnimationFrame flow that defers work until the next frame.

- `this` binding for function listeners and `window` global property reflection
  - Needed so `this.getAttribute(...)` works in listeners and `window.hashApi` is readable as `hashApi`.

## Missing runtime capabilities

- `Intl.*`
  - Blocks `tests/integration_cases/issue_167_finitefield_site_regressions.rs`
  - Blocks `tests/integration_cases/issue_171_finitefield_site_regressions.rs`
  - Blocks `tests/integration_cases/issue_173_finitefield_site_regressions.rs`
  - Blocks `tests/integration_cases/issue_174_175_finitefield_site_regressions.rs`
  - Blocks `tests/integration_cases/issue_199_finitefield_site_regression.rs`
  - Blocks the first two tests in `tests/integration_cases/issue_212_finitefield_site_runtime_regressions.rs`

- `DOMParser` and `XMLSerializer`
  - Blocks `tests/integration_cases/issue_170_finitefield_site_regressions.rs`
  - Blocks `tests/integration_cases/issue_181_finitefield_site_regressions.rs`
  - Blocks `tests/integration_cases/issue_183_184_finitefield_site_regressions.rs`

- `Worker`, `Blob`, `URL.createObjectURL`, and `URL.revokeObjectURL`
  - Blocks the worker regression in `tests/integration_cases/regression_runtime_state_fixes.rs`

- `TextEncoder` and `TextDecoder`
  - Blocks the text-encoding regression in `tests/integration_cases/regression_runtime_state_fixes.rs`

- Typed arrays and typed-array helpers
  - Blocks `tests/integration_cases/typed_array_from_map_fn.rs`
  - Blocks the digest/bytes test in `tests/integration_cases/issue_202_finitefield_site_regressions.rs`

- `CSS.escape`
  - Blocks `tests/integration_cases/css_escape_global.rs`

- `crypto.subtle`
  - Blocks `tests/integration_cases/issue_202_finitefield_site_regressions.rs`

- Keyboard event construction helpers
  - Blocks `dispatch_keyboard`-style tests in `tests/integration_cases/open_issue_regressions.rs`
  - Blocks keyboard-dispatch tests in `tests/integration_cases/issue_138_141_finitefield_site_regressions.rs`
  - Blocks the keyboard-specific regression coverage in `tests/integration_cases/regression_parser_fixes.rs`

- Layout-dependent sticky positioning
  - Blocks `tests/integration_cases/issue_203_finitefield_site_regressions.rs`

## Missing test hooks

- Timer queue inspection helpers such as `pending_timers` and `run_due_timers`
  - The Rust runtime regression that checks timer overflow introspection cannot be ported verbatim yet.

- Clipboard error-injection hooks
  - The Rust contract test that toggles clipboard write failures has no direct Zig equivalent yet.

- Richer file-input mock payloads
  - Rust can seed file names plus contents; Zig currently records file names for `setFiles`.

- High-level navigation page stubs
  - Rust tests can inject a page for a future location via a helper; Zig currently exposes direct location mocks instead.

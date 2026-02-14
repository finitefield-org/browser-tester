# browser-tester

A deterministic, browser-like test runtime implemented in pure Rust.

- Japanese README: [translations/ja/README.md](translations/ja/README.md)
- Developed by [Finite Field, K.K.](https://finitefield.org)
- Detailed design source: `doc/e2e-lite-runtime-design.md`

## Purpose

`browser-tester` is built for fast, deterministic DOM/script testing without spinning up a real browser.

It is designed to:
- Run in-process Rust tests with predictable behavior.
- Validate form flows and UI logic quickly.
- Avoid external browser, WebDriver, and Node.js dependencies.

## Quick Start

```rust
use browser_tester::Harness;

fn main() -> browser_tester::Result<()> {
    let html = r#"
      <input id='name' />
      <button id='submit'>Submit</button>
      <p id='result'></p>
      <script>
        document.getElementById('submit').addEventListener('click', () => {
          const name = document.getElementById('name').value;
          document.getElementById('result').textContent = `Hello, ${name}`;
        });
      </script>
    "#;

    let mut h = Harness::from_html(html)?;
    h.type_text("#name", "Alice")?;
    h.click("#submit")?;
    h.assert_text("#result", "Hello, Alice")?;
    Ok(())
}
```

Run tests:

```bash
cargo test
```

## Runtime Policy

- `eval` is intentionally not implemented (security + determinism).
- Time APIs are fake-clock based:
  - `Date.now()`
  - `performance.now()`

## Browser-like Mocks

The runtime supports controlled mocks for browser APIs:

- `fetch`
  - `Harness::set_fetch_mock(url, body)`
  - `Harness::clear_fetch_mocks()`
  - `Harness::take_fetch_calls()`
- `matchMedia`
  - `Harness::set_match_media_mock(query, matches)`
  - `Harness::clear_match_media_mocks()`
  - `Harness::set_default_match_media_matches(matches)`
  - `Harness::take_match_media_calls()`
- `alert` / `confirm` / `prompt`
  - `Harness::take_alert_messages()`
  - `Harness::enqueue_confirm_response(bool)`
  - `Harness::set_default_confirm_response(bool)`
  - `Harness::enqueue_prompt_response(Option<&str>)`
  - `Harness::set_default_prompt_response(Option<&str>)`

## Design Highlights

This section is a readable summary of `doc/e2e-lite-runtime-design.md`.

### Scope

In scope:
- Parse one HTML string and build an in-memory DOM.
- Execute inline `<script>` only.
- Simulate DOM interactions and events for tests.

Out of scope:
- External CSS/JS loading.
- Real network I/O (only mocked `fetch` is supported).
- Rendering/layout engine behavior.
- iframe, shadow DOM, custom elements.

### Architecture

Main components:
- `dom_core`: DOM tree, selectors, attributes/properties.
- `script_runtime`: custom parser + evaluator (JS subset).
- `event_system`: capture/target/bubble propagation and default actions.
- `runtime_core`: initialization, script execution, timer/microtask queues.
- `test_harness`: high-level APIs used by tests.

### DOM Model

- Arena-like node storage (`Vec<Node>`) with `NodeId`.
- Node types: `Document`, `Element`, `Text`.
- Element state includes `value`, `checked`, `disabled`, `readonly`, `required`.
- ID index stores duplicates: `HashMap<String, Vec<NodeId>>`.
  - `#id` / `getElementById` return the first match.

### Selector Support (Highlights)

- Basic: `#id`, `.class`, `tag`, `*`, attributes.
- Combinators: descendant, child (`>`), adjacent sibling (`+`), general sibling (`~`).
- Pseudo-classes include:
  - Structural: `:first-child`, `:last-child`, `:only-child`, `:empty`, `:nth-*`.
  - State: `:checked`, `:disabled`, `:enabled`, `:required`, `:optional`, `:read-only`, `:read-write`, `:focus`, `:focus-within`, `:active`.
  - Functional: `:not(...)`, `:is(...)`, `:where(...)`, `:has(...)`.
- Attribute operators: `=`, `^=`, `$=`, `*=`, `~=`, `|=`.
- Unsupported selectors fail explicitly (no silent fallback).

### Script Runtime Support (Highlights)

- Control flow: `if/else`, `while`, `do...while`, `for`, `for...in`, `for...of`, `break`, `continue`, `return`.
- Common expressions/operators, including compound assignments and BigInt basics.
- DOM APIs used in form/UI tests.
- FormData subset:
  - `new FormData(form)`
  - `.get`, `.has`, `.getAll`, `.append`
- Timers and task APIs:
  - `setTimeout`, `setInterval`, `clearTimeout`, `clearInterval`
  - `requestAnimationFrame`, `cancelAnimationFrame`
  - `queueMicrotask`

### Events

- Event phases: capture -> target -> bubble.
- Supports:
  - `preventDefault`
  - `stopPropagation`
  - `stopImmediatePropagation`
- Event fields include `type`, `target`, `currentTarget`, `eventPhase`, `timeStamp`, `defaultPrevented`, `isTrusted`.

### Deterministic Time and Queues

- Fake clock starts at `0ms`.
- Timer controls:
  - `advance_time(ms)`
  - `advance_time_to(target_ms)`
  - `run_due_timers()`
  - `run_next_timer()` / `run_next_due_timer()`
  - `flush()`
- Queue controls:
  - `clear_timer(id)`
  - `clear_all_timers()`
  - `pending_timers()`
- Safety:
  - `set_timer_step_limit(max_steps)` to cap task execution loops.

### Tracing and Debugging

- `enable_trace(true)` enables trace collection.
- Category toggles:
  - `set_trace_events(bool)`
  - `set_trace_timers(bool)`
- Output/log controls:
  - `set_trace_stderr(bool)`
  - `set_trace_log_limit(max_entries)`
  - `take_trace_logs()`
- DOM snapshot helper:
  - `dump_dom(selector)`

### Error Model

`Error` variants:
- `HtmlParse`
- `ScriptParse`
- `ScriptRuntime`
- `SelectorNotFound`
- `UnsupportedSelector`
- `TypeMismatch`
- `AssertionFailed` (includes DOM snippet)

### Harness API Surface (Core)

```rust
impl Harness {
    pub fn from_html(html: &str) -> Result<Self>;

    // Actions
    pub fn type_text(&mut self, selector: &str, text: &str) -> Result<()>;
    pub fn set_checked(&mut self, selector: &str, checked: bool) -> Result<()>;
    pub fn click(&mut self, selector: &str) -> Result<()>;
    pub fn focus(&mut self, selector: &str) -> Result<()>;
    pub fn blur(&mut self, selector: &str) -> Result<()>;
    pub fn submit(&mut self, selector: &str) -> Result<()>;
    pub fn dispatch(&mut self, selector: &str, event: &str) -> Result<()>;

    // Assertions
    pub fn assert_text(&self, selector: &str, expected: &str) -> Result<()>;
    pub fn assert_value(&self, selector: &str, expected: &str) -> Result<()>;
    pub fn assert_checked(&self, selector: &str, expected: bool) -> Result<()>;
    pub fn assert_exists(&self, selector: &str) -> Result<()>;

    // Time and queue
    pub fn now_ms(&self) -> i64;
    pub fn advance_time(&mut self, ms: i64) -> Result<()>;
    pub fn advance_time_to(&mut self, target_ms: i64) -> Result<()>;
    pub fn flush(&mut self) -> Result<()>;

    // Diagnostics
    pub fn dump_dom(&self, selector: &str) -> Result<String>;
    pub fn take_trace_logs(&mut self) -> Vec<String>;
}
```

## Notes

- This README is a concise, readable summary.
- For full implementation-level detail, use `doc/e2e-lite-runtime-design.md`.

# browser-tester

A deterministic browser-like testing crate implemented entirely in Rust.

- Japanese README: [translations/ja/README.md](translations/ja/README.md)
- Architecture: [doc/architecture.md](doc/architecture.md)
- Capability matrix: [doc/capability-matrix.md](doc/capability-matrix.md)
- Mock guide: [doc/mock-guide.md](doc/mock-guide.md)
- HTML spec conformance roadmap: [doc/html-spec-conformance-roadmap.md](doc/html-spec-conformance-roadmap.md)
- Developed by [Finite Field, K.K.](https://finitefield.org)

## Purpose

- Run DOM and script tests deterministically in a single process.
- Test browser-like interactions without an external browser, WebDriver, or Node.js.
- Keep time, randomness, and test-only browser APIs under Rust-side control.

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

    let mut harness = Harness::from_html(html)?;
    harness.type_text("#name", "Alice")?;
    harness.click("#submit")?;
    harness.assert_text("#result", "Hello, Alice")?;
    Ok(())
}
```

## Core API

Constructors:

- `Harness::from_html(html)`
- `Harness::from_html_with_url(url, html)`
- `Harness::from_html_with_local_storage(html, &[("key", "value"), ...])`
- `Harness::from_html_with_url_and_local_storage(url, html, &[("key", "value"), ...])`

Actions:

- `type_text`
- `set_select_value`
- `set_input_files`
- `set_checked`
- `click`
- `press_enter`
- `copy`
- `paste`
- `cut`
- `focus`
- `blur`
- `submit`
- `dispatch`
- `dispatch_keyboard`

Assertions:

- `assert_text`
- `assert_value`
- `assert_checked`
- `assert_exists`
- `dump_dom`

Time and scheduling:

- `now_ms`
- `advance_time`
- `advance_time_to`
- `run_due_timers`
- `run_next_timer`
- `run_next_due_timer`
- `flush`
- `pending_timers`
- `clear_timer`
- `clear_all_timers`

Trace and diagnostics:

- `enable_trace`
- `take_trace_logs`
- `set_trace_stderr`
- `set_trace_events`
- `set_trace_timers`
- `set_trace_log_limit`

## Test Mocks

Main mock families:

- `fetch`
- `confirm` / `prompt` / `alert`
- `window.print()`
- `location` and history-backed mock pages
- `navigator.clipboard`
- `localStorage` seed state
- download capture
- `input[type="file"]`
- `matchMedia`

Full API details and examples are in [doc/mock-guide.md](doc/mock-guide.md).

Minimal fetch mock example:

```rust
use browser_tester::Harness;

fn main() -> browser_tester::Result<()> {
    let html = r#"
      <button id='run'>run</button>
      <p id='out'></p>
      <script>
        document.getElementById('run').addEventListener('click', () => {
          fetch('https://app.local/api/message')
            .then((res) => res.text())
            .then((text) => {
              document.getElementById('out').textContent = text;
            });
        });
      </script>
    "#;

    let mut h = Harness::from_html(html)?;
    h.set_fetch_mock("https://app.local/api/message", "hello");
    h.click("#run")?;
    h.assert_text("#out", "hello")?;
    assert_eq!(
        h.take_fetch_calls(),
        vec!["https://app.local/api/message".to_string()]
    );
    Ok(())
}
```

## Runtime Constraints

- `eval` is intentionally not implemented.
- `Date.now()` and `performance.now()` are backed by a fake clock.
- `Math.random()` is deterministic and can be reseeded with `set_random_seed`.
- Real network I/O is not the target. `fetch` should be used with mocks.
- Rendering, layout, style application, and the accessibility tree are intentionally partial.
- This crate provides a deterministic browser-like subset, not full browser compatibility.

Form submission policy:

- `Harness::submit(selector)` follows a user-like submission path.
- Script-side `form.requestSubmit([submitter])` also follows the user-like path.
- Script-side `form.submit()` bypasses validation and does not dispatch `submit`.

## Running Tests

Run the main test suite:

```bash
cargo test
```

Run by test layer:

```bash
scripts/run-test-layer.sh contract
scripts/run-test-layer.sh subsystem dom_navigation_dialog
scripts/run-test-layer.sh integration issue_174_175
scripts/run-test-layer.sh fuzz
```

Run repository maintenance guardrails:

```bash
scripts/check-file-size-guard.sh
```

Run the Go workspace checklist:

```bash
bash scripts/run-go-checklist.sh
```

Regular integration tests are grouped under a single `integration_suite` target to reduce
Cargo's per-file test crate and link overhead:

```bash
cargo test --test integration_suite issue_174_175
```

Minimal public contract coverage for stable `Harness` APIs:

```bash
cargo test --test contract_harness_core
```

If you only want to confirm the contract target still builds:

```bash
scripts/run-test-layer.sh contract-build
```

Property and fuzz tests for parser and runtime:

```bash
# default: parser=256 cases, runtime=128 cases
cargo test --test parser_property_fuzz_test --test runtime_property_fuzz_test -- --nocapture

# quick profile
BROWSER_TESTER_PROPTEST_CASES=64 \
BROWSER_TESTER_RUNTIME_PROPTEST_CASES=64 \
cargo test --test parser_property_fuzz_test --test runtime_property_fuzz_test

# deep profile
BROWSER_TESTER_PROPTEST_CASES=1024 \
BROWSER_TESTER_RUNTIME_PROPTEST_CASES=512 \
cargo test --test parser_property_fuzz_test --test runtime_property_fuzz_test
```

- `BROWSER_TESTER_PROPTEST_CASES`: default parser-oriented case count
- `BROWSER_TESTER_RUNTIME_PROPTEST_CASES`: runtime action case count
- shrunk failing seeds are stored in:
  - `tests/proptest-regressions/parser_property_fuzz_test.txt`
  - `tests/proptest-regressions/runtime_property_fuzz_test.txt`

## Documentation

- Architecture and subsystem overview: [doc/architecture.md](doc/architecture.md)
- Capability classification: [doc/capability-matrix.md](doc/capability-matrix.md)
- File-size guard and allowlist policy: [doc/file-size-guard.md](doc/file-size-guard.md)
- Public API checklist: [doc/public-api-checklist.md](doc/public-api-checklist.md)
- Subsystem ownership map: [doc/subsystem-map.md](doc/subsystem-map.md)
- Test taxonomy and placement rules: [doc/test-taxonomy.md](doc/test-taxonomy.md)
- Mock APIs and examples: [doc/mock-guide.md](doc/mock-guide.md)
- HTML conformance roadmap: [doc/html-spec-conformance-roadmap.md](doc/html-spec-conformance-roadmap.md)
- WPT audit inventory: [doc/p3-wpt-audit-inventory.md](doc/p3-wpt-audit-inventory.md)

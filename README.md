# browser-tester

A pure Rust crate for deterministic browser-like testing in Rust.

- Japanese translation: `translations/ja/README.md`
- Design doc: `doc/e2e-lite-runtime-design.md`

## Purpose

- Provide an in-process, deterministic runtime for DOM and script testing.
- Enable browser interaction tests without external browsers, WebDriver, or Node.js.

## Usage

1. Create a test harness from HTML.
2. Interact with elements using selectors.
3. Assert the resulting DOM state.

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

Run tests:

```bash
cargo test
```

## Runtime Policy

- `eval` is intentionally not implemented to keep the runtime secure and deterministic.
- Time support is based on the fake clock and exposed via `Date.now()` and `performance.now()`.

## Test Mocks

- `fetch` is designed to be replaced with mocks in tests.
- `confirm` / `prompt` provide APIs to inject mock return values.
- Common APIs:
  - `Harness::set_fetch_mock(url, body)`
  - `Harness::enqueue_confirm_response(bool)`
  - `Harness::enqueue_prompt_response(Option<&str>)`

Developed by [Finite Field, K.K.](https://finitefield.org).

# browser-tester Mock Guide

## Overview

`browser-tester` treats test mocks as first-class runtime features.
The goal is to let tests control browser-like behavior deterministically without relying on a real browser environment.

This document groups the main mock families and shows representative usage.

## Common Pattern

Most mock-backed tests follow the same shape:

1. Build a `Harness`
2. Seed deterministic mock state
3. Trigger a user-like action
4. Assert DOM state or inspect captured artifacts

```rust
use browser_tester::Harness;

fn main() -> browser_tester::Result<()> {
    let html = "<button id='run'>run</button>";
    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    Ok(())
}
```

## Fetch

Main APIs:

- `set_fetch_mock(url, body)`
- `set_fetch_mock_response(url, status, body)`
- `clear_fetch_mocks()`
- `take_fetch_calls()`

Example:

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

Use `set_fetch_mock_response` when status code matters:

```rust
h.set_fetch_mock_response("https://app.local/api/message", 404, "missing");
```

## Dialogs and Print

Main APIs:

- `enqueue_confirm_response(bool)`
- `set_default_confirm_response(bool)`
- `enqueue_prompt_response(Option<&str>)`
- `set_default_prompt_response(Option<&str>)`
- `take_alert_messages()`
- `take_print_call_count()`

Example:

```rust
use browser_tester::Harness;

fn main() -> browser_tester::Result<()> {
    let html = r#"
      <button id='run'>run</button>
      <p id='out'></p>
      <script>
        document.getElementById('run').addEventListener('click', () => {
          const accepted = confirm('continue?');
          const answer = prompt('name?');
          document.getElementById('out').textContent =
            `${accepted}:${answer ?? 'null'}`;
          alert('done');
          window.print();
        });
      </script>
    "#;

    let mut h = Harness::from_html(html)?;
    h.enqueue_confirm_response(true);
    h.enqueue_prompt_response(Some("Alice"));
    h.click("#run")?;
    h.assert_text("#out", "true:Alice")?;
    assert_eq!(h.take_alert_messages(), vec!["done".to_string()]);
    assert_eq!(h.take_print_call_count(), 1);
    Ok(())
}
```

## Location and History-Backed Mock Pages

Main APIs:

- `set_location_mock_page(url, html)`
- `clear_location_mock_pages()`
- `take_location_navigations()`
- `location_reload_count()`

Example:

```rust
use browser_tester::{Harness, LocationNavigation, LocationNavigationKind};

fn main() -> browser_tester::Result<()> {
    let html = r#"
      <button id='go'>go</button>
      <script>
        document.getElementById('go').addEventListener('click', () => {
          location.assign('https://app.local/next');
        });
      </script>
    "#;

    let mut h = Harness::from_html(html)?;
    h.set_location_mock_page("https://app.local/next", "<p id='msg'>next page</p>");
    h.click("#go")?;
    h.assert_text("#msg", "next page")?;

    assert_eq!(
        h.take_location_navigations(),
        vec![LocationNavigation {
            kind: LocationNavigationKind::Assign,
            from: "about:blank".to_string(),
            to: "https://app.local/next".to_string(),
        }]
    );
    Ok(())
}
```

For `history.go(0)`, `history.back()`, or `history.forward()`, provide deterministic pages for the URLs that may be visited:

```rust
let mut h = Harness::from_html(html)?;
h.set_location_mock_page("about:blank", "<p id='marker'>reloaded</p>");
h.click("#run")?;
h.assert_text("#marker", "reloaded")?;
```

## Clipboard

Main APIs:

- `set_clipboard_text(text)`
- `clipboard_text()`
- `set_clipboard_read_error(Some(name))`
- `set_clipboard_write_error(Some(name))`
- `clear_clipboard_errors()`
- `take_clipboard_writes()`
- `copy(selector)`
- `paste(selector)`
- script-side override via `navigator.clipboard = { ... }`

Read example:

```rust
use browser_tester::Harness;

fn main() -> browser_tester::Result<()> {
    let html = r#"
      <button id='run'>run</button>
      <p id='out'></p>
      <script>
        document.getElementById('run').addEventListener('click', () => {
          navigator.clipboard.readText().then((text) => {
            document.getElementById('out').textContent = text;
          });
        });
      </script>
    "#;

    let mut h = Harness::from_html(html)?;
    h.set_clipboard_text("seeded");
    h.click("#run")?;
    h.assert_text("#out", "seeded")?;
    Ok(())
}
```

Binary write capture example:

```rust
use browser_tester::{ClipboardPayloadArtifact, ClipboardWriteArtifact, Harness};

fn main() -> browser_tester::Result<()> {
    let html = r#"
      <button id='run'>run</button>
      <script>
        document.getElementById('run').addEventListener('click', async () => {
          const pngBlob = new Blob([new Uint8Array([137, 80, 78, 71, 1, 2, 3])], {
            type: 'image/png'
          });
          await navigator.clipboard.write([
            new ClipboardItem({ 'image/png': pngBlob })
          ]);
        });
      </script>
    "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    assert_eq!(
        h.take_clipboard_writes(),
        vec![ClipboardWriteArtifact {
            payloads: vec![ClipboardPayloadArtifact {
                mime_type: "image/png".to_string(),
                bytes: vec![137, 80, 78, 71, 1, 2, 3],
            }],
        }]
    );
    Ok(())
}
```

Script-side override example:

```rust
use browser_tester::Harness;

fn main() -> browser_tester::Result<()> {
    let html = r#"
      <button id='run'>run</button>
      <p id='out'></p>
      <script>
        navigator.clipboard = {
          readText: () => Promise.resolve('stubbed-read'),
          writeText: () => Promise.resolve('stubbed-write'),
        };
        document.getElementById('run').addEventListener('click', () => {
          navigator.clipboard.writeText('x')
            .then(() => navigator.clipboard.readText())
            .then((text) => {
              document.getElementById('out').textContent = text;
            });
        });
      </script>
    "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#out", "stubbed-read")?;
    Ok(())
}
```

## localStorage Seed State

Main APIs:

- `Harness::from_html_with_local_storage(...)`
- `Harness::from_html_with_url_and_local_storage(...)`

Example:

```rust
use browser_tester::Harness;

fn main() -> browser_tester::Result<()> {
    let html = r#"
      <p id='out'></p>
      <script>
        document.getElementById('out').textContent =
          localStorage.getItem('token') || 'missing';
      </script>
    "#;

    let h = Harness::from_html_with_local_storage(html, &[("token", "seeded-token")])?;
    h.assert_text("#out", "seeded-token")?;
    Ok(())
}
```

`window.localStorage` is also assignable inside script when a local stub is more convenient than Rust-side seeding.

## Download Capture

Main APIs:

- `take_downloads()`

Download capture works for deterministic flows such as `Blob` + `URL.createObjectURL()` + `<a download>.click()`.

Example:

```rust
use browser_tester::{DownloadArtifact, Harness};

fn main() -> browser_tester::Result<()> {
    let html = r#"
      <button id='run'>run</button>
      <script>
        document.getElementById('run').addEventListener('click', () => {
          const blob = new Blob(['a,b\n1,2\n'], { type: 'text/csv;charset=utf-8;' });
          const url = URL.createObjectURL(blob);
          const a = document.createElement('a');
          a.href = url;
          a.download = 'report.csv';
          a.click();
          URL.revokeObjectURL(url);
        });
      </script>
    "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    assert_eq!(
        h.take_downloads(),
        vec![DownloadArtifact {
            filename: Some("report.csv".to_string()),
            mime_type: Some("text/csv;charset=utf-8;".to_string()),
            bytes: b"a,b\n1,2\n".to_vec(),
        }]
    );
    Ok(())
}
```

## File Inputs

Main APIs:

- `set_input_files(selector, &[MockFile { ... }, ...])`
- `MockFile::new(name)`

Behavior:

- when selection changes: dispatch `input` then `change`
- when selection does not change: dispatch `cancel`
- for non-`multiple` inputs, only the first mock file is selected
- mocked files expose `arrayBuffer()` / `text()` / `bytes()` / `stream()`
- mocked image files can be consumed by `createImageBitmap(file)`

Example:

```rust
use browser_tester::{Harness, MockFile};

fn main() -> browser_tester::Result<()> {
    let html = r#"
      <input id='upload' type='file' multiple required>
      <button id='run'>run</button>
      <p id='out'></p>
      <script>
        const input = document.getElementById('upload');
        document.getElementById('run').addEventListener('click', () => {
          const files = input.files;
          document.getElementById('out').textContent =
            input.value + ':' + files.length + ':' + files.map((f) => f.name).join(',');
        });
      </script>
    "#;

    let mut h = Harness::from_html(html)?;
    h.set_input_files(
        "#upload",
        &[
            MockFile::new("first.txt").with_text("hello"),
            MockFile {
                name: "nested/second.txt".to_string(),
                size: 7,
                mime_type: "text/plain".to_string(),
                last_modified: 99,
                webkit_relative_path: "nested/second.txt".to_string(),
                bytes: b"second!".to_vec(),
            },
        ],
    )?;
    h.click("#run")?;
    h.assert_text("#out", "C:\\fakepath\\first.txt:2:first.txt,second.txt")?;
    Ok(())
}
```

## matchMedia

Main APIs:

- `set_match_media_mock(query, matches)`
- `set_default_match_media_matches(matches)`
- `clear_match_media_mocks()`
- `take_match_media_calls()`

Example:

```rust
use browser_tester::Harness;

fn main() -> browser_tester::Result<()> {
    let html = r#"
      <button id='run'>run</button>
      <p id='out'></p>
      <script>
        document.getElementById('run').addEventListener('click', () => {
          const result = window.matchMedia('(prefers-reduced-motion: reduce)');
          document.getElementById('out').textContent = String(result.matches);
        });
      </script>
    "#;

    let mut h = Harness::from_html(html)?;
    h.set_match_media_mock("(prefers-reduced-motion: reduce)", true);
    h.click("#run")?;
    h.assert_text("#out", "true")?;
    assert_eq!(
        h.take_match_media_calls(),
        vec!["(prefers-reduced-motion: reduce)".to_string()]
    );
    Ok(())
}
```

## Related Documents

- Architecture overview: [architecture.md](architecture.md)
- Public package README: [../README.md](../README.md)

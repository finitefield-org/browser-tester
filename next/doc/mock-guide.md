# Mock Guide

Mocks are part of the intended core product surface for `next/`.
Even in Phase 0, the workspace already reserves explicit families so runtime behavior can land without growing `Harness` into a giant bag of `set_*` methods.

## Current Mock Families

- `fetch`
- `dialogs`
- `clipboard`
- `location`
- `downloads`
- `file_input`
- `storage`

They are exposed from the public facade through `Harness::mocks_mut()`.

## Public Mock Actions

Phase 4 adds thin public actions on `Harness` for the mock families that need to behave like browser services:

- `fetch(url)`
- `alert(message)`
- `confirm(message)`
- `prompt(message)`
- `read_clipboard()`
- `write_clipboard(text)`
- `navigate(url)`
- `set_files(selector, files)`
- `capture_download(file_name, bytes)`

The typed registry is still the source of truth for seeds and capture.
Use `Harness::mocks_mut()` to configure the family, then call the matching action on `Harness`.
Download capture records `DownloadCapture` artifacts in the registry and exposes them through `downloads().artifacts()`.

## Minimal Example

```rust
use browser_tester_next::Harness;

fn main() -> browser_tester_next::Result<()> {
    let mut harness = Harness::from_html("<input id='upload' type='file'>")?;

    harness
        .mocks_mut()
        .fetch()
        .respond_text("https://app.local/api/message", 200, "ok");
    harness.mocks_mut().dialogs().push_confirm(true);
    harness.mocks_mut().clipboard().seed_text("copied text");

    let response = harness.fetch("https://app.local/api/message")?;
    assert_eq!(response.status, 200);
    assert_eq!(response.body, "ok");
    harness.alert("Notice")?;
    assert!(harness.confirm("Continue?")?);
    assert_eq!(harness.read_clipboard()?, "copied text");
    harness.write_clipboard("copied text")?;
    harness.set_files("#upload", ["report.csv"])?;
    harness.capture_download("report.csv", b"downloaded bytes".to_vec())?;
    harness.navigate("https://app.local/next")?;

    assert_eq!(harness.mocks_mut().fetch().calls().len(), 1);
    assert_eq!(harness.mocks_mut().dialogs().alert_messages().len(), 1);
    assert_eq!(harness.mocks_mut().dialogs().confirm_messages().len(), 1);
    assert_eq!(harness.mocks_mut().clipboard().writes().len(), 1);
    assert_eq!(harness.mocks_mut().location().navigations().len(), 1);
    assert_eq!(harness.mocks_mut().file_input().selections().len(), 1);
    {
        let downloads = harness.mocks_mut().downloads();
        assert_eq!(downloads.artifacts().len(), 1);
        assert_eq!(downloads.artifacts()[0].bytes, b"downloaded bytes".to_vec());
    }
    Ok(())
}
```

## Failure Example

```rust
use browser_tester_next::Harness;

fn main() -> browser_tester_next::Result<()> {
    let mut harness = Harness::builder().build()?;

    let fetch_error = harness
        .fetch("https://app.local/api/missing")
        .expect_err("missing fetch mock should fail");
    assert!(fetch_error.to_string().contains("no fetch mock configured"));

    let confirm_error = harness
        .confirm("Continue?")
        .expect_err("confirm should require a queued response");
    assert!(confirm_error
        .to_string()
        .contains("confirm() requires a queued response"));

    let clipboard_error = harness
        .read_clipboard()
        .expect_err("clipboard reads should require a seed");
    assert!(clipboard_error
        .to_string()
        .contains("clipboard text has not been seeded"));

    let download_error = harness
        .capture_download(" ", b"downloaded bytes".to_vec())
        .expect_err("blank download names should fail");
    assert!(download_error
        .to_string()
        .contains("capture_download() requires a non-empty file name"));

    Ok(())
}
```

## Design Rules Per Mock Family

Each family is expected to support:

- response injection or seed state
- failure injection where applicable
- call capture or artifact capture
- reset semantics

Examples:

- `fetch`: response rules, error rules, request call capture
- `dialogs`: queued confirm/prompt answers, alert capture, and call-message capture
- `clipboard`: seeded read state and write capture
- `location`: current URL seed and navigation capture
- `downloads`: artifact capture through the registry and `Harness::capture_download(...)`
- `file_input`: file selection seed and capture

## Why the Registry Shape Matters

The rewrite intentionally avoids letting `Harness` grow into hundreds of one-off mock methods.
A typed registry keeps the public facade small while still making deterministic hooks discoverable.

## Planned Documentation Bar

Whenever a new test-only mock becomes public, its docs should ship with:

- the public API shape
- a minimal success example
- a failure-path example
- an explanation of call capture or artifact capture
- README updates
- this guide updated in the same change

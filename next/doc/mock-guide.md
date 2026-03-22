# Mock Guide

Mocks are part of the intended core product surface for `next/`.
Even in Phase 0, the workspace already reserves explicit families so runtime behavior can land without growing `Harness` into a giant bag of `set_*` methods.

## Current Mock Families

- `fetch`
- `dialogs`
- `clipboard`
- `print`
- `open`
- `close`
- `scroll`
- `location`
- `downloads`
- `file_input`
- `matchMedia`
- `storage`

They are exposed from the public facade through `Harness::mocks_mut()`. `matchMedia` is configured through the builder seed API and the registry, then consumed from scripts via `window.matchMedia(...)`. The dialogs family is consumed from scripts via `window.alert(...)`, `window.confirm(...)`, and `window.prompt(...)`, with alert/confirm/prompt message capture recorded in the registry.

## Public Mock Actions

Phase 4 adds thin public actions on `Harness` for the mock families that need to behave like browser services:

- `fetch(url)`
- `alert(message)`
- `confirm(message)`
- `prompt(message)`
- `read_clipboard()`
- `write_clipboard(text)`
- `print()`
- `open(url)`
- `close()`
- `scroll_to(x, y)`
- `scroll_by(dx, dy)`
- `navigate(url)`
- `set_files(selector, files)`
- `capture_download(file_name, bytes)`

The typed registry is still the source of truth for seeds and capture.
Use `Harness::mocks_mut()` to configure the family, then call the matching action on `Harness`.
Download capture records `DownloadCapture` artifacts in the registry and exposes them through `downloads().artifacts()`.
`matchMedia` is registry-backed and builder-seeded rather than a standalone `Harness` action.
The location family also captures script-side `window.location.assign()`, `window.location.replace()`, `window.location.reload()`, and `window.location.hash` / `document.location.hash` / `window.location.pathname` / `document.location.pathname` / `window.location.search` / `document.location.search` assignments through the same navigation log that `Harness::navigate()` uses.
The same navigation log also covers `document.location.href`, `document.location.hash`, `document.location.pathname`, `document.location.search`, `document.location.origin`, `window.location.href`, `window.location.hash`, `window.location.pathname`, `window.location.search`, and `window.location.origin` assignments.

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
    harness.print()?;
    harness.open("https://app.local/popup")?;
    harness.close()?;
    harness.scroll_to(0, 120)?;
    harness.set_files("#upload", ["report.csv"])?;
    harness.capture_download("report.csv", b"downloaded bytes".to_vec())?;
    harness.navigate("https://app.local/next")?;

    assert_eq!(harness.mocks_mut().fetch().calls().len(), 1);
    assert_eq!(harness.mocks_mut().dialogs().alert_messages().len(), 1);
    assert_eq!(harness.mocks_mut().dialogs().confirm_messages().len(), 1);
    assert_eq!(harness.mocks_mut().clipboard().writes().len(), 1);
    assert_eq!(harness.mocks_mut().print().calls().len(), 1);
    assert_eq!(harness.mocks_mut().open().calls().len(), 1);
    assert_eq!(harness.mocks_mut().close().calls().len(), 1);
    assert_eq!(harness.mocks_mut().scroll().calls().len(), 1);
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

`matchMedia` is configured through the builder seed API and inspected through the registry:

```rust
use browser_tester_next::Harness;

fn main() -> browser_tester_next::Result<()> {
    let harness = Harness::builder()
        .html("<main id='out'></main><script>const list = window.matchMedia('(prefers-color-scheme: dark)'); document.getElementById('out').textContent = String(list.matches) + ':' + list.media;</script>")
        .match_media([("(prefers-color-scheme: dark)", true)])
        .build()?;

    assert_eq!(harness.mocks_mut().match_media().calls().len(), 1);
    harness.assert_text("#out", "true:(prefers-color-scheme: dark)")?;
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

    let print_error = Harness::builder()
        .print_failure("print blocked")
        .html("<script>window.print();</script>")
        .build()
        .expect_err("print failure should fail bootstrap when window.print runs");
    assert!(print_error.to_string().contains("print blocked"));

    let open_error = Harness::builder()
        .open_failure("popup blocked")
        .html("<script>window.open('https://app.local/popup');</script>")
        .build()
        .expect_err("open failure should fail bootstrap when window.open runs");
    assert!(open_error.to_string().contains("popup blocked"));

    let scroll_error = Harness::builder()
        .scroll_failure("scroll blocked")
        .html("<script>window.scrollTo(0, 120);</script>")
        .build()
        .expect_err("scroll failure should fail bootstrap when window.scrollTo runs");
    assert!(scroll_error.to_string().contains("scroll blocked"));

    let close_error = Harness::builder()
        .close_failure("window closed")
        .html("<script>window.close();</script>")
        .build()
        .expect_err("close failure should fail bootstrap when window.close runs");
    assert!(close_error.to_string().contains("window closed"));

    let match_media_error = Harness::builder()
        .html("<script>window.matchMedia('(prefers-color-scheme: dark)').matches;</script>")
        .build()
        .expect_err("unseeded matchMedia should fail");
    assert!(match_media_error
        .to_string()
        .contains("no matchMedia mock configured for `(prefers-color-scheme: dark)`"));

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
- `location`: current URL seed and navigation capture, including `window.location.href`, `document.location.href`, `window.location.hash`, `document.location.hash`, `window.location.pathname`, `document.location.pathname`, `window.location.search`, `document.location.search`, `window.location.origin`, `document.location.origin`, `window.location.assign()`, `window.location.replace()`, and `window.location.reload()`
- `downloads`: artifact capture through the registry and `Harness::capture_download(...)`
- `file_input`: file selection seed and capture
- `print`: call capture through the registry and `Harness::print(...)`, plus optional builder-seeded bootstrap failure
- `open`: call capture through the registry and `Harness::open(...)`, plus optional builder-seeded bootstrap failure for `window.open(...)`; the mock returns `undefined` rather than a popup `WindowProxy`
- `close`: call capture through the registry and `Harness::close(...)`, plus optional builder-seeded bootstrap failure for `window.close(...)`
- `scroll`: call capture through the registry and `Harness::scroll_to(...)` / `Harness::scroll_by(...)`, plus optional builder-seeded bootstrap failure for `window.scrollTo(...)` / `window.scrollBy(...)`
- `matchMedia`: query seed state and call capture for `window.matchMedia(...)`

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

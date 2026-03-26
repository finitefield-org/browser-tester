# browser-tester

`browser-tester` is a clean-room rewrite workspace for a deterministic browser-test harness.
The repository is split into explicit subsystems so `Harness` stays a thin facade and each capability can be added as a small, testable slice.

## Scope

- HTML behavior follows the bounded subset documented under `html-standard/`.
- Legacy or deprecated HTML features are intentionally out of scope.
- Rendering, layout, and live network I/O are out of scope; tests rely on deterministic mocks.

## Current State

- The Rust workspace lives under `crates/`.
- Core subsystems are split across `browser-tester`, `bt-dom`, `bt-runtime`, and `bt-script`.
- The repo already covers the main deterministic test surfaces: DOM parsing, bounded selectors, inline script execution, events and forms, mock-driven browser services, and DOM mutation/reflection slices.
- The current support level for each capability is tracked in [Capability Matrix](doc/capability-matrix.md).
- The placement rules and implementation order are documented in [Subsystem Map](doc/subsystem-map.md) and [Implementation Guide](doc/implementation-guide.md).

## Workspace Layout

```text
crates/
  browser-tester/   # public facade crate (`browser_tester`)
  bt-dom/           # DOM store, HTML parser, selector subset
  bt-runtime/       # session, scheduler, mocks, debug state
  bt-script/        # script runtime and host-binding seam
doc/
  architecture.md
  capability-matrix.md
  implementation-guide.md
  mock-guide.md
  limitations.md
  subsystem-map.md
  roadmap.md
  adr/
html-standard/
```

## Quick Start

```bash
cargo test
```

## Examples

### DOM and Events

```rust
use browser_tester::Harness;

fn main() -> browser_tester::Result<()> {
    let harness = Harness::from_html(
        r#"
        <form id="profile">
          <input id="name">
          <input id="agree" type="checkbox">
          <button id="submit" type="submit">Save</button>
        </form>
        <div id="out"></div>
        <script>
          document.getElementById('profile').addEventListener('submit', () => {
            document.getElementById('out').textContent =
              document.getElementById('name').value + ':' +
              String(document.getElementById('agree').checked);
          });
        </script>
        "#,
    )?;

    harness.type_text("#name", "Alice")?;
    harness.click("#agree")?;
    harness.click("#submit")?;
    harness.assert_checked("#agree", true)?;
    harness.assert_text("#out", "Alice:true")?;
    Ok(())
}
```

### Deterministic Mocks

```rust
use browser_tester::{FileInputFile, Harness};

fn main() -> browser_tester::Result<()> {
    let mut harness = Harness::from_html("<input id='upload' type='file'>")?;

    harness
        .mocks_mut()
        .fetch()
        .respond_text("https://app.local/api/message", 200, "ok");
    harness.mocks_mut().dialogs().push_confirm(true);
    harness.mocks_mut().clipboard().seed_text("seeded");

    let response = harness.fetch("https://app.local/api/message")?;
    assert_eq!(response.body, "ok");
    assert!(harness.confirm("Continue?")?);
    assert_eq!(harness.read_clipboard()?, "seeded");

    harness.set_files(
        "#upload",
        [FileInputFile::from_text("report.csv", "name,age\nAda,42").with_mime_type("text/csv")],
    )?;
    harness.capture_download("report.csv", b"downloaded bytes".to_vec())?;
    Ok(())
}
```

`set_files()` still accepts plain file names for compatibility, but `FileInputFile` lets you seed bytes and MIME type when the script under test reads `input.files[0].text()`, `input.files.item(0).text()`, or `await input.files[0].text()` in async code. `input.files` and `event.dataTransfer.files` are exposed as deterministic FileList-like collections with indexed access and `item()`. Use `FileInputFile::with_read_error(...)` when you need `FileReader` to fire deterministic `error` / `loadend` callbacks, and use `dispatch_drop()` with the same payload type when the code path reads `event.dataTransfer.files`.

For the full mock surface and failure-path examples, see [Mock Guide](doc/mock-guide.md).

## Docs

- [Architecture](doc/architecture.md) - target architecture and seams
- [Subsystem Map](doc/subsystem-map.md) - where code should live
- [Implementation Guide](doc/implementation-guide.md) - recommended build order
- [Capability Matrix](doc/capability-matrix.md) - current support level and scope
- [Mock Guide](doc/mock-guide.md) - deterministic mock families and usage
- [Limitations](doc/limitations.md) - explicit out-of-scope behavior
- [Roadmap](doc/roadmap.md) - staged implementation plan
- [Publication Checklist](doc/publish-checklist.md) - release gate

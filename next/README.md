# browser-tester next

This directory is a clean-room rewrite workspace for `browser-tester`.

It is intentionally organized around the staged plan from [`next.md`](../next.md):

- split the runtime into explicit subsystems
- keep `Harness` as a thin public facade
- treat deterministic mocks as first-class APIs
- keep the initial surface small and documented

Current status:

- a compilable Rust workspace exists under `next/crates/`
- `HarnessBuilder`, `Session`, `DomStore`, scheduler, mock registry, and error taxonomy are in place
- Phase 1 DOM parsing, selector subset support, `assert_exists`, and debug DOM dumps are implemented
- Phase 2 inline script bootstrapping, `document.getElementById(...).textContent = ...`, and listener registration are implemented
- Phase 3 event dispatch, ancestor bubbling, cancelable click default actions, form controls, `click`, `type_text`, `set_checked`, `set_select_value`, `focus`, `blur`, `submit`, `dispatch`, `assert_value`, and `assert_checked` are implemented
- Phase 4 fake clock hardening, microtask semantics, and runtime mock wiring for fetch, dialogs, clipboard, location, and file input are implemented

Workspace layout:

```text
next/
  crates/
    browser-tester/   # public facade crate (`browser_tester_next`)
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
```

Quick start:

```bash
cd next
cargo test
```

Minimal Phase 3 example:

```rust
use browser_tester_next::Harness;

fn main() -> browser_tester_next::Result<()> {
    let harness = Harness::from_html(
        "<form id='profile'><input id='name'><input id='agree' type='checkbox'><button id='submit' type='submit'>Save</button></form><div id='out'></div><script>document.getElementById('profile').addEventListener('submit', () => { document.getElementById('out').textContent = document.getElementById('name').value + ':' + String(document.getElementById('agree').checked); });</script>",
    )?;

    harness.type_text("#name", "Alice")?;
    harness.click("#agree")?;
    harness.click("#submit")?;
    harness.assert_checked("#agree", true)?;
    harness.assert_text("#out", "Alice:true")?;
    Ok(())
}
```

Minimal Phase 4 mock example:

```rust
use browser_tester_next::Harness;

fn main() -> browser_tester_next::Result<()> {
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

    harness.set_files("#upload", ["report.csv"])?;
    harness.navigate("https://app.local/next")?;
    Ok(())
}
```

Design docs:

- [Architecture](doc/architecture.md)
- [Capability Matrix](doc/capability-matrix.md)
- [Implementation Guide](doc/implementation-guide.md)
- [Mock Guide](doc/mock-guide.md)
- [Limitations](doc/limitations.md)
- [Subsystem Map](doc/subsystem-map.md)
- [Roadmap](doc/roadmap.md)

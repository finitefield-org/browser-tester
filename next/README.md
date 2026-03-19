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
- event dispatch, forms, and the wider action surface are still gated for later phases

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

Minimal Phase 2 example:

```rust
use browser_tester_next::Harness;

fn main() -> browser_tester_next::Result<()> {
    let harness = Harness::from_html(
        "<main id='app'></main><script>document.getElementById('app').textContent = 'Hello';</script>",
    )?;

    harness.assert_exists("#app")?;
    assert_eq!(
        harness.debug().dump_dom(),
        "#document\n  <main id=\"app\">\n    \"Hello\"\n  </main>\n  <script>\n    \"document.getElementById('app').textContent = 'Hello';\"\n  </script>"
    );
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

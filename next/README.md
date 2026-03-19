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
- events, forms, and script execution are still gated for later phases

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

Minimal Phase 1 example:

```rust
use browser_tester_next::Harness;

fn main() -> browser_tester_next::Result<()> {
    let harness = Harness::from_html(
        "<main id='app'><span data-state='ready'>Hello</span></main>",
    )?;

    harness.assert_exists("#app")?;
    assert_eq!(
        harness.debug().dump_dom(),
        "#document\n  <main id=\"app\">\n    <span data-state=\"ready\">\n      \"Hello\"\n    </span>\n  </main>"
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

# browser-tester next

This directory is a clean-room rewrite workspace for `browser-tester`.

It is intentionally starting at Phase 0 from [`next.md`](../next.md):

- split the runtime into explicit subsystems
- keep `Harness` as a thin public facade
- treat deterministic mocks as first-class APIs
- keep the initial surface small and documented

Current status:

- a compilable Rust workspace exists under `next/crates/`
- `HarnessBuilder`, `Session`, `DomStore`, scheduler, mock registry, and error taxonomy skeletons are in place
- DOM parsing, selectors, events, and script execution are not implemented yet

Workspace layout:

```text
next/
  crates/
    browser-tester/   # public facade crate (`browser_tester_next`)
    bt-dom/           # DOM store and HTML bootstrap skeleton
    bt-runtime/       # session, scheduler, mocks, debug state
    bt-script/        # script runtime and host-binding skeleton
  doc/
    architecture.md
    capability-matrix.md
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

Minimal Phase 0 example:

```rust
use browser_tester_next::Harness;

fn main() -> browser_tester_next::Result<()> {
    let harness = Harness::builder()
        .url("https://app.local/")
        .local_storage([("token", "abc")])
        .build()?;

    assert_eq!(harness.debug().url(), "https://app.local/");
    assert_eq!(harness.debug().source_html(), None);
    Ok(())
}
```

Design docs:

- [Architecture](doc/architecture.md)
- [Capability Matrix](doc/capability-matrix.md)
- [Mock Guide](doc/mock-guide.md)
- [Limitations](doc/limitations.md)
- [Subsystem Map](doc/subsystem-map.md)
- [Roadmap](doc/roadmap.md)


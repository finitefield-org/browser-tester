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

## Phase 0 Example

```rust
use browser_tester_next::Harness;

fn main() -> browser_tester_next::Result<()> {
    let mut harness = Harness::builder().build()?;

    harness
        .mocks_mut()
        .fetch()
        .respond_text("https://app.local/api/message", 200, "ok");
    harness
        .mocks_mut()
        .fetch()
        .fail("https://app.local/api/error", "network disabled");
    harness.mocks_mut().dialogs().push_confirm(true);
    harness.mocks_mut().clipboard().seed_text("copied text");
    harness
        .mocks_mut()
        .file_input()
        .set_files("#upload", ["report.csv"]);
    harness
        .mocks_mut()
        .downloads()
        .capture("report.csv", b"id,name\n1,Alice\n".to_vec());

    assert_eq!(harness.mocks_mut().fetch().responses().len(), 1);
    assert_eq!(harness.mocks_mut().fetch().errors().len(), 1);
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
- `dialogs`: queued confirm/prompt answers, alert capture
- `clipboard`: seeded read state and write capture
- `location`: current URL seed and navigation capture
- `downloads`: artifact capture
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


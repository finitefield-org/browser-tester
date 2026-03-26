# [P1] Add FileReader support for file import flows

## Problem
Real fixtures read uploaded files with `new FileReader()` and `readAsArrayBuffer()` / `readAsText()`, but the runtime has no `FileReader` implementation. Without it, file import handlers cannot run even if the selected file object is available.

## Evidence
- `tests/fixtures/csv-deduplicator-inline-script.html` uses `new FileReader()` and `reader.readAsArrayBuffer(file)`.
- The runtime source has no `FileReader`, `readAsText`, or `readAsArrayBuffer` implementation.

## Expected behavior
- `FileReader` can read seeded file payloads deterministically.
- `load`, `error`, and `loadend` callbacks are invoked in order.
- `result` is populated with `ArrayBuffer` or text as appropriate.
- Errors can be injected for unreadable payloads.

## Suggested regression tests
- Read a seeded CSV payload via `readAsArrayBuffer()` and assert the parsed text.
- Read a seeded text payload via `readAsText()` and assert `result`.
- Verify the error path fires when payload is missing or unreadable.
